import Vue from 'vue'
import * as k8s from '@kubernetes/client-node'
import * as net from 'net'
import killable from 'killable'
import * as Sentry from '@sentry/electron'

import { patchForward } from '../../lib/k8s-port-forwarding-patch'
import * as resourceKinds from '../../lib/constants/workload-types'
import { createToolset } from '../helpers/validations'
import * as connectionStates from '../../lib/constants/connection-states'
import { k8nApiPrettyError } from '../../lib/helpers/k8n-api-error'
import { netServerPrettyError } from '../../lib/helpers/net-server-error'
import { getServiceLabel } from '../../lib/helpers/service'
import { buildKubeConfig } from '../../lib/helpers/cluster'
import { buildSentryIgnoredError } from '../../lib/errors'

// Schema of an item
const { validate } = createToolset({
  type: 'object',
  required: ['port', 'serviceId', 'state', 'flags'],
  properties: {
    port: { type: 'integer', minimum: 0, maximum: 65535 },
    serviceId: { type: 'string' },
    state: { type: 'string', enum: Object.values(connectionStates) },
    flags: {
      type: 'object',
      required: ['http'],
      properties: {
        http: { type: 'boolean' }
      }
    }
  }
})

// See the schema above
const state = {}

const mutations = {
  SET(state, payloadedItem) {
    const item = { flags: { http: false }, ...payloadedItem }

    const valid = validate(item)
    if (valid) Vue.set(state, [item.address, item.port].join(':'), item)
    else throw new Error(JSON.stringify(validate.errors))
  },

  SET_FLAG(state, { address, port, flagName, flagValue }) {
    if (!port) throw new Error('port must be present')
    if (!flagName) throw new Error('flagName must be present')
    const key = [address, port].join(':')
    if (state[key]) {
      Vue.set(state[key].flags, flagName, flagValue)
    }
  },

  DELETE(state, { address, port }) {
    if (!port) throw new Error('port must present')
    Vue.delete(state, [address, port].join(':'))
  }
}

const servers = {}

function killServer(commit, address, port) {
  let called = false
  const serverKey = [address, port].join(':')
  const server = servers[serverKey]

  const onClose = () => {
    if (!called) {
      console.info(`Port ${serverKey} have freed`)
      commit('DELETE', { address, port })
      delete servers[serverKey]
      called = true
    }
  }

  if (server) {
    server.kill(onClose)

    // if there wasn't connections, server closed immediately without emitting callback
    // so I have to call callback manually
    if (!server.listening) onClose()
  } else {
    onClose()
  }
}

async function startForward(commit, k8sForward, service, target) {
  const server = net.createServer(function(socket) {
    k8sForward.portForward(target.namespace, target.podName, [target.remotePort], socket, null, socket, 3)
    k8sForward.disconnectOnErr = false
  })

  killable(server)

  const resultPromise = new Promise((resolve) => {
    const serviceString = `Service ${getServiceLabel(service)}(${service.id})`
    const localAddress = service.localAddress || 'localhost'

    server.on('error', (error) => {
      if (server.listening) {
        killServer(commit, localAddress, target.localPort)
      } else {
        server.kill()
        const prettyError = netServerPrettyError(error)
        console.info(`Error while forwarding ${serviceString}: ${prettyError.message}`)
        resolve({ success: false, error: prettyError })
      }
    })

    server.listen(target.localPort, localAddress, () => {
      const serverKey = [localAddress, target.localPort].join(':')
      servers[serverKey] = server
      commit('SET', {
        address: localAddress,
        port: target.localPort,
        serviceId: service.id,
        state: connectionStates.CONNECTED
      })
      resolve({ success: true })
    })
  })

  resultPromise.then(result => result.success && updateFlags(commit, service, target))

  return resultPromise
}

function updateFlags(commit, service, target) {
  return updateHttpFlag(commit, service, target)
}

async function updateHttpFlag(commit, service, target) {
  const controller = new AbortController()
  setTimeout(() => controller.abort(), 5000)

  try {
    const localAddress = service.localAddress || 'localhost'
    await fetch(
      `http://${localAddress}:${target.localPort}`,
      { signal: controller.signal, method: 'OPTIONS' })
    commit('SET_FLAG', { address: localAddress, port: target.localPort, flagName: 'http', flagValue: true })
  } catch (e) {}
}

function prepareK8sToolsWithCluster(cluster) {
  let kubeConfig

  try {
    kubeConfig = buildKubeConfig(cluster.config)
  } catch (error) {
    const message = typeof error.message === 'string'
      ? `\nError message:\n---\n${error.message.substr(0, 1000)}`
      : null
    throw buildSentryIgnoredError(`Cluster config is invalid.${message}`)
  }

  const k8sPortForward = new k8s.PortForward(kubeConfig)
  patchForward(k8sPortForward)

  return { k8sPortForward, kubeConfig }
}

function prepareK8sToolsWithService(rootState, service) {
  const cluster = rootState.Clusters.items[service.clusterId]
  if (!cluster) return { success: false, message: `Cluster(id=${service.clusterId}) doesn't exist` }

  return prepareK8sToolsWithCluster(cluster)
}

async function loadResource(kubeConfig, service) {
  const { workloadType: resourceKind, workloadName: resourceName, namespace } = service

  switch (resourceKind) {
    case resourceKinds.POD:
      return loadPod(kubeConfig, namespace, resourceName)

    case resourceKinds.DEPLOYMENT:
      return loadDeployment(kubeConfig, namespace, resourceName)

    case resourceKinds.SERVICE:
      return loadService(kubeConfig, namespace, resourceName)

    default:
      throw new Error(`Unacceptable resourceKind=${resourceKind}`)
  }
}

async function loadPod(kubeConfig, namespace, podName) {
  const coreApi = kubeConfig.makeApiClient(k8s.CoreV1Api)

  try {
    return (await coreApi.readNamespacedPod(podName, namespace)).body
  } catch (error) {
    throw k8nApiPrettyError(error, { _object: `Pod "${podName}"` })
  }
}

async function loadDeployment(kubeConfig, namespace, deploymentName) {
  const coreApi = kubeConfig.makeApiClient(k8s.CoreV1Api)

  try {
    return (await coreApi.readNamespacedDeployment(deploymentName, namespace)).body
  } catch (error) {
    throw k8nApiPrettyError(error, { _object: `Deployment "${deploymentName}"` })
  }
}

async function loadService(kubeConfig, namespace, serviceName) {
  const coreApi = kubeConfig.makeApiClient(k8s.CoreV1Api)

  try {
    return (await coreApi.readNamespacedService(serviceName, namespace)).body
  } catch (error) {
    throw k8nApiPrettyError(error, { _object: `Service "${serviceName}"` })
  }
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
      const localPort = forward.localPort
      return { namespace, localPort, remotePort, podName: pod.metadata.name }
    }
    default:
      throw new Error(`Unacceptable resource.kind=${resource.kind}`)
  }
}

async function getPodNameFromDeployment(kubeConfig, deployment) {
  const coreApi = kubeConfig.makeApiClient(k8s.CoreV1Api)

  const { metadata: { namespace, name }, spec: { selector: { matchLabels } } } = deployment
  const matchLabelKey = Object.keys(matchLabels)[0]
  const labelSelector = `${matchLabelKey}=${matchLabels[matchLabelKey]}`

  const { body: podsBody } = await coreApi.listNamespacedPod(namespace, null, null, null, null, labelSelector)
  const podName = podsBody.items.length && podsBody.items[0].metadata.name
  if (!podName) throw buildSentryIgnoredError(`There are no pods in '${name}' deployment.`)

  return podName
}

async function getPodFromService(kubeConfig, service) {
  const coreApi = kubeConfig.makeApiClient(k8s.CoreV1Api)

  const { metadata: { name, namespace }, spec: { selector } } = service
  if (!selector) throw buildSentryIgnoredError(`Service '${name}' does not have a selector.`)

  const { body: pods } = await coreApi.listNamespacedPod(namespace, null, null, null, null, stringifySelector(selector))
  const pod = pods.items.length && pods.items[0]
  if (!pod) throw buildSentryIgnoredError(`There are no pods in '${name}' service.`)

  return pod
}

function stringifySelector(selector) {
  const strings = []
  for (const key of Object.keys(selector)) {
    strings.push(`${key}=${selector[key]}`)
  }
  return strings.join(',')
}

function mapServicePort(service, port, pod) {
  let targetPort = null

  for (const servicePort of service.spec.ports) {
    if (servicePort.port === port) {
      targetPort = servicePort.targetPort
      break
    }
  }

  if (typeof targetPort === 'number') {
    return targetPort
  } else if (typeof targetPort === 'string') {
    for (const container of pod.spec.containers) {
      for (const containerPort of container.ports) {
        if (containerPort.name === targetPort) return containerPort.containerPort
      }
    }
  }

  throw buildSentryIgnoredError(
    `Service "${
      service.metadata.name
    }" does not have a service port ${port}. Available ports: ${
      service.spec.ports.map(x => x.port).join(', ')
    }`)
}

function createConnectingStates(commit, service) {
  for (const forward of service.forwards) {
    const address = service.localAddress || 'localhost'
    const port = forward.localPort
    commit('SET', { address, port, serviceId: service.id, state: connectionStates.CONNECTING })
  }
}

function clearStates(commit, service) {
  for (const forward of service.forwards) {
    commit('DELETE', { address: service.localAddress || 'localhost', port: forward.localPort })
  }
}

function validateThatRequiredPortsFree(state, service) {
  const localAddress = service.localAddress || 'localhost';
  for (const forward of service.forwards) {
    if (state[[localAddress, forward.localPort].join(':')]) {
      throw buildSentryIgnoredError(`Port ${localAddress}:${forward.localPort} is busy.`)
    }
  }
}

let actions = {
  async createConnection({ commit, state, rootState }, service) {
    try {
      validateThatRequiredPortsFree(state, service)
      createConnectingStates(commit, service)

      const { kubeConfig, k8sPortForward } = prepareK8sToolsWithService(rootState, service)
      const resource = await loadResource(kubeConfig, service)
      const results = await Promise.all(service.forwards.map(async forward => {
        const target = await getTarget(kubeConfig, resource, forward)
        const result = await startForward(commit, k8sPortForward, service, target)
        return { ...result, service, forward, target }
      }))

      const success = !results.find(x => !x.success)
      if (!success) {
        for (const result of results) {
          killServer(commit, result.service.localAddress || 'localhost', result.target.localPort)
        }
      }

      return { success, results }
    } catch (error) {
      // TODO a breadcrumb for originError
      Sentry.captureException(error)
      clearStates(commit, service)
      return { success: false, error }
    }
  },

  deleteConnection({ commit }, service) {
    for (const forward of service.forwards) {
      killServer(commit, service.localAddress || 'localhost', forward.localPort)
    }
  }
}

export default {
  persisted: false,
  namespaced: true,
  state,
  mutations,
  actions
}
