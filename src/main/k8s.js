// All Kubernetes + port-forwarding logic. Runs in the MAIN process only.
// The renderer reaches this exclusively through IPC (see ipc.js / preload).
import * as net from 'net'
import {
  KubeConfig,
  CoreV1Api,
  AppsV1Api,
  PortForward
} from '@kubernetes/client-node'

import * as configStoringMethods from '../renderer/lib/constants/config-storing-methods'
import * as resourceKinds from '../renderer/lib/constants/workload-types'
import * as connectionStates from '../renderer/lib/constants/connection-states'

// ---------------------------------------------------------------------------
// KubeConfig
// ---------------------------------------------------------------------------

// Throws on invalid config — callers handle it.
export function buildKubeConfig(clusterConfig) {
  const kubeConfig = new KubeConfig()

  if (clusterConfig.storingMethod === configStoringMethods.PATH) {
    kubeConfig.loadFromFile(clusterConfig.path)
    kubeConfig.setCurrentContext(clusterConfig.currentContext)
  } else if (clusterConfig.storingMethod === configStoringMethods.CONTENT) {
    kubeConfig.loadFromString(clusterConfig.content)
  } else {
    throw new Error(`storingMethod "${clusterConfig.storingMethod}" is invalid.`)
  }

  return kubeConfig
}

// Returns a plain, IPC-serializable error object or null (Error objects don't
// survive `ipcRenderer.invoke`, so custom fields like `originError` are flattened).
function prettyK8sError(error, objectLabel) {
  const status = error?.code ?? error?.statusCode ?? error?.response?.statusCode
  let bodyMessage
  const body = error?.body
  if (body && typeof body === 'object') {
    bodyMessage = body.message
  } else if (typeof body === 'string') {
    try { bodyMessage = JSON.parse(body).message } catch { /* not JSON */ }
  }

  let message
  if (objectLabel && status === 404) message = `${objectLabel} not found.`
  else if (objectLabel && status === 403) message = `${objectLabel} forbidden.`
  else message = bodyMessage || error?.message || (objectLabel ? `${objectLabel} can't be fetched.` : 'Request failed.')

  return { message, originMessage: error?.message || bodyMessage || null }
}

export async function checkConnection(clusterConfig, context = null) {
  let kubeConfig
  try {
    kubeConfig = buildKubeConfig(clusterConfig)
  } catch (error) {
    return prettyK8sError(error)
  }

  if (context) kubeConfig.setCurrentContext(context)

  try {
    await kubeConfig.makeApiClient(CoreV1Api).listNode()
    return null
  } catch (error) {
    return prettyK8sError(error)
  }
}

function serializeContexts(kubeConfig) {
  return {
    contexts: kubeConfig.contexts.map(c => ({ name: c.name, cluster: c.cluster, user: c.user })),
    currentContext: kubeConfig.getCurrentContext()
  }
}

export function contextsFromString(content) {
  try {
    const kubeConfig = new KubeConfig()
    kubeConfig.loadFromString(content)
    return serializeContexts(kubeConfig)
  } catch (error) {
    return { error: error.message }
  }
}

export function contextsFromFile(filePath) {
  try {
    const kubeConfig = new KubeConfig()
    kubeConfig.loadFromFile(filePath)
    return serializeContexts(kubeConfig)
  } catch (error) {
    return { error: error.message }
  }
}

// ---------------------------------------------------------------------------
// Resource listing (namespace / resource-name autocompletes in ServiceForm)
// ---------------------------------------------------------------------------

export async function listNamespaces(clusterConfig) {
  const kubeConfig = buildKubeConfig(clusterConfig)
  const coreApi = kubeConfig.makeApiClient(CoreV1Api)
  const result = await coreApi.listNamespace()
  return result.items.map(x => x.metadata.name)
}

export async function listResources(clusterConfig, kind, namespace) {
  const kubeConfig = buildKubeConfig(clusterConfig)

  if (kind === resourceKinds.POD) {
    const coreApi = kubeConfig.makeApiClient(CoreV1Api)
    const result = await coreApi.listNamespacedPod({ namespace })
    return result.items.map(x => x.metadata.name)
  } else if (kind === resourceKinds.DEPLOYMENT) {
    const appsApi = kubeConfig.makeApiClient(AppsV1Api)
    const result = await appsApi.listNamespacedDeployment({ namespace })
    return result.items.map(x => x.metadata.name)
  } else if (kind === resourceKinds.SERVICE) {
    const coreApi = kubeConfig.makeApiClient(CoreV1Api)
    const result = await coreApi.listNamespacedService({ namespace })
    return result.items.map(x => x.metadata.name)
  }

  return []
}

// ---------------------------------------------------------------------------
// Resource resolution: map a saved service to a concrete pod + remote port
// ---------------------------------------------------------------------------

async function loadResource(kubeConfig, service) {
  const { workloadType: kind, workloadName: name, namespace } = service
  const coreApi = kubeConfig.makeApiClient(CoreV1Api)

  try {
    switch (kind) {
      case resourceKinds.POD:
        return await coreApi.readNamespacedPod({ name, namespace })
      case resourceKinds.SERVICE:
        return await coreApi.readNamespacedService({ name, namespace })
      case resourceKinds.DEPLOYMENT: {
        const appsApi = kubeConfig.makeApiClient(AppsV1Api)
        return await appsApi.readNamespacedDeployment({ name, namespace })
      }
      default:
        throw new Error(`Unacceptable resourceKind=${kind}`)
    }
  } catch (error) {
    const label = `${kind[0].toUpperCase()}${kind.slice(1)} "${name}"`
    const pretty = prettyK8sError(error, label)
    throw new Error(pretty.message)
  }
}

function stringifySelector(selector) {
  return Object.keys(selector).map(key => `${key}=${selector[key]}`).join(',')
}

async function getPodNameFromDeployment(kubeConfig, deployment) {
  const coreApi = kubeConfig.makeApiClient(CoreV1Api)
  const { metadata: { namespace, name }, spec: { selector: { matchLabels } } } = deployment
  const labelSelector = stringifySelector(matchLabels)

  const pods = await coreApi.listNamespacedPod({ namespace, labelSelector })
  const podName = pods.items.length && pods.items[0].metadata.name
  if (!podName) throw new Error(`There are no pods in '${name}' deployment.`)
  return podName
}

async function getPodFromService(kubeConfig, service) {
  const coreApi = kubeConfig.makeApiClient(CoreV1Api)
  const { metadata: { name, namespace }, spec: { selector } } = service
  if (!selector) throw new Error(`Service '${name}' does not have a selector.`)

  const pods = await coreApi.listNamespacedPod({ namespace, labelSelector: stringifySelector(selector) })
  const pod = pods.items.length && pods.items[0]
  if (!pod) throw new Error(`There are no pods in '${name}' service.`)
  return pod
}

function mapServicePort(service, port, pod) {
  let targetPort = null
  for (const servicePort of service.spec.ports) {
    if (servicePort.port === port) { targetPort = servicePort.targetPort; break }
  }

  if (typeof targetPort === 'number') return targetPort
  if (typeof targetPort === 'string') {
    for (const container of pod.spec.containers) {
      for (const containerPort of container.ports || []) {
        if (containerPort.name === targetPort) return containerPort.containerPort
      }
    }
  }

  throw new Error(
    `Service "${service.metadata.name}" does not have a service port ${port}. ` +
    `Available ports: ${service.spec.ports.map(x => x.port).join(', ')}`
  )
}

async function getTarget(kubeConfig, resource, forward) {
  const { name, namespace } = resource.metadata

  switch (resource.kind) {
    case 'Pod':
      return { namespace, ...forward, podName: name }
    case 'Deployment': {
      const podName = await getPodNameFromDeployment(kubeConfig, resource)
      return { namespace, ...forward, podName }
    }
    case 'Service': {
      const pod = await getPodFromService(kubeConfig, resource)
      const remotePort = mapServicePort(resource, forward.remotePort, pod)
      return { namespace, localPort: forward.localPort, remotePort, podName: pod.metadata.name }
    }
    default:
      throw new Error(`Unacceptable resource.kind=${resource.kind}`)
  }
}

// readNamespaced* responses omit `kind`; getTarget switches on it, so set it.
function resourceKindLabel(workloadType) {
  if (workloadType === resourceKinds.POD) return 'Pod'
  if (workloadType === resourceKinds.DEPLOYMENT) return 'Deployment'
  if (workloadType === resourceKinds.SERVICE) return 'Service'
  return null
}

// ---------------------------------------------------------------------------
// Port-forward lifecycle (local TCP servers ↔ k8s PortForward WebSockets)
// ---------------------------------------------------------------------------

// serverKey ("address:port") -> { server, sockets:Set }
const servers = {}

// Set by ipc.js; pushes connection state to the renderer's vuex store.
let emit = () => {}
export function setEmitter(fn) { emit = fn }

function netServerPrettyError(error) {
  if (error.code === 'EADDRINUSE') return `Port ${error.port} already in use`
  if (error.code === 'EACCES') return "Application hasn't enough privileges to use local ports below 1024."
  return error.message
}

function killServer(address, port) {
  const serverKey = [address, port].join(':')
  const entry = servers[serverKey]

  const onClose = () => {
    if (servers[serverKey]) {
      delete servers[serverKey]
      emit('DELETE', { address, port })
      console.info(`Port ${serverKey} has been freed`)
    }
  }

  if (entry) {
    for (const socket of entry.sockets) socket.destroy()
    entry.server.close(onClose)
    if (!entry.server.listening) onClose()
  } else {
    onClose()
  }
}

function startForward(k8sForward, service, target) {
  const localAddress = service.localAddress || 'localhost'
  const serverKey = [localAddress, target.localPort].join(':')
  const sockets = new Set()

  const server = net.createServer(socket => {
    sockets.add(socket)
    socket.on('close', () => sockets.delete(socket))
    k8sForward.portForward(target.namespace, target.podName, [target.remotePort], socket, null, socket, 3)
    k8sForward.disconnectOnErr = false
  })

  return new Promise(resolve => {
    server.on('error', error => {
      if (server.listening) {
        killServer(localAddress, target.localPort)
      } else {
        try { server.close() } catch { /* not listening */ }
        const message = netServerPrettyError(error)
        console.info(`Error while forwarding service ${service.id}: ${message}`)
        resolve({ success: false, error: { message, originMessage: error.message } })
      }
    })

    server.listen(target.localPort, localAddress, () => {
      servers[serverKey] = { server, sockets }
      emit('SET', {
        address: localAddress,
        port: target.localPort,
        serviceId: service.id,
        state: connectionStates.CONNECTED
      })
      resolve({ success: true })
      updateHttpFlag(service, target)
    })
  })
}

async function updateHttpFlag(service, target) {
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), 5000)
  try {
    const localAddress = service.localAddress || 'localhost'
    await fetch(`http://${localAddress}:${target.localPort}`, { signal: controller.signal, method: 'OPTIONS' })
    emit('SET_FLAG', { address: localAddress, port: target.localPort, flagName: 'http', flagValue: true })
  } catch { /* not http, ignore */ } finally { clearTimeout(timer) }
}

function validatePortsFree(service) {
  const localAddress = service.localAddress || 'localhost'
  for (const forward of service.forwards) {
    if (servers[[localAddress, forward.localPort].join(':')]) {
      throw new Error(`Port ${localAddress}:${forward.localPort} is busy.`)
    }
  }
}

function emitConnectingStates(service) {
  const address = service.localAddress || 'localhost'
  for (const forward of service.forwards) {
    emit('SET', { address, port: forward.localPort, serviceId: service.id, state: connectionStates.CONNECTING })
  }
}

function clearStates(service) {
  const address = service.localAddress || 'localhost'
  for (const forward of service.forwards) emit('DELETE', { address, port: forward.localPort })
}

export async function createConnection({ service, cluster }) {
  try {
    if (!cluster) throw new Error(`Cluster(id=${service.clusterId}) doesn't exist`)
    validatePortsFree(service)
    emitConnectingStates(service)

    const kubeConfig = buildKubeConfig(cluster.config)
    const k8sForward = new PortForward(kubeConfig)

    const resource = await loadResource(kubeConfig, service)
    resource.kind = resource.kind || resourceKindLabel(service.workloadType)

    const results = await Promise.all(service.forwards.map(async forward => {
      const target = await getTarget(kubeConfig, resource, forward)
      const result = await startForward(k8sForward, service, target)
      return { ...result, forward, target }
    }))

    const success = !results.find(x => !x.success)
    if (!success) {
      for (const result of results) killServer(service.localAddress || 'localhost', result.target.localPort)
    }

    // Strip the (non-serializable-safe) target before returning to renderer.
    const slim = results.map(({ success, error, forward }) => ({ success, error, forward }))
    return { success, results: slim }
  } catch (error) {
    clearStates(service)
    return { success: false, error: { message: error.message, originMessage: error.message } }
  }
}

export function deleteConnection(service) {
  for (const forward of service.forwards) {
    killServer(service.localAddress || 'localhost', forward.localPort)
  }
}
