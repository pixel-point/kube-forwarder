import Vue from 'vue'
import * as k8s from '@kubernetes/client-node'
import * as net from 'net'
import killable from 'killable'

import { patchForward } from '../../lib/k8s-port-forwarding-patch'
import * as workloadTypes from '../../lib/constants/workload-types'
import { createToolset } from '../helpers/validations'
import * as connectionStates from '../../lib/constants/connection-states'
import { k8nApiPrettyError } from '../../lib/helpers/k8n-api-error'
import { netServerPrettyError } from '../../lib/helpers/net-server-error'
import { getServiceLabel } from '../../lib/helpers/service'
import { isWebDemo } from '../../lib/environment'

const { validate } = createToolset({
  type: 'object',
  required: ['port', 'serviceId', 'state'],
  properties: {
    port: { type: 'integer', minimum: 0, maximum: 65535 },
    serviceId: { type: 'string' },
    state: { type: 'string', enum: Object.values(connectionStates) }
  }
})

const state = {
  // <port>: { port, serviceId, state }
  // state - one of 'connected', 'connecting'
}

const mutations = {
  SET(state, item) {
    const valid = validate(item)
    if (valid) Vue.set(state, item.port, item)
    else throw new Error(JSON.stringify(validate.errors))
  },
  DELETE(state, port) {
    if (!port) throw new Error('port must present')
    Vue.set(state, port)
  }
}

const servers = {}

function killServer(commit, port) {
  let called = false
  const server = servers[port]

  const onClose = () => {
    if (!called) {
      console.info(`Port ${port} have freed`)
      commit('DELETE', port)
      delete servers[port]
      called = true
    }
  }

  if (server) {
    server.kill(onClose())

    // if there wasn't connections, server closed immediately without emitting callback
    // so I have to call callback manually
    if (!server.listening) onClose()
  } else {
    onClose()
  }
}

// function getPodName(kservice)

async function startForward(commit, k8sForward, service, forward, podName) {
  const listenPort = forward.localPort
  const server = net.createServer(function(socket) {
    k8sForward.portForward(service.namespace, podName, [forward.remotePort], socket, null, socket, 3)
    k8sForward.disconnectOnErr = false
  })

  killable(server)
  return new Promise((resolve) => {
    server.on('error', (error) => {
      if (server.listening) {
        killServer(commit, listenPort)
      } else {
        server.kill()
        const prettyError = netServerPrettyError(error)
        console.info(`Error while forwarding Service ${getServiceLabel(service)}(${service.id}): ${prettyError.message}`)
        resolve({ success: false, error: prettyError, service, forward })
      }
    })

    server.on('listening', () => {
      servers[listenPort] = server
      commit('SET', { port: listenPort, serviceId: service.id, state: connectionStates.CONNECTED })
      console.info(`Service ${getServiceLabel(service)}(${service.id}) is forwarding port ${listenPort} to ${podName}:${forward.remotePort}`)
      resolve({ success: true, service, forward })
    })

    server.listen(listenPort, '127.0.0.1')
  })
}

function prepareK8sToolsWithCluster(cluster) {
  const kubeConfig = new k8s.KubeConfig()

  try {
    kubeConfig.loadFromString(cluster.config)
  } catch (error) {
    throw new Error('Cluster config is invalid.')
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

async function getPodName(kubeConfig, service) {
  const { workloadType, workloadName } = service

  switch (workloadType) {
    case workloadTypes.POD:
      await validatePodName(kubeConfig, service.namespace, workloadName)
      return workloadName
    case workloadTypes.DEPLOYMENT:
      return getPodNameFromDeployment(kubeConfig, service.namespace, workloadName)
    default:
      throw new Error(`Unacceptable workloadType=${workloadType}`)
  }
}

async function validatePodName(kubeConfig, namespace, podName) {
  const coreApi = kubeConfig.makeApiClient(k8s.Core_v1Api)

  try {
    await coreApi.readNamespacedPod(podName, namespace)
  } catch (error) {
    throw k8nApiPrettyError(error, { _object: `Pod "${podName}"` })
  }
}

async function getPodNameFromDeployment(kubeConfig, namespace, deploymentName) {
  const coreApi = kubeConfig.makeApiClient(k8s.Core_v1Api)
  const extensionsApi = kubeConfig.makeApiClient(k8s.Extensions_v1beta1Api)

  let deployment
  try {
    deployment = (await extensionsApi.readNamespacedDeployment(deploymentName, namespace)).body
  } catch (error) {
    throw k8nApiPrettyError(error, { _object: `Deployment "${deploymentName}"` })
  }

  const { matchLabels } = deployment.spec.selector
  const matchLabelKey = Object.keys(matchLabels)[0]
  const labelSelector = `${matchLabelKey}=${matchLabels[matchLabelKey]}`

  const { body: podsBody } = await coreApi.listNamespacedPod(namespace, null, null, null, null, labelSelector)
  const podName = podsBody.items.length && podsBody.items[0].metadata.name
  if (!podName) throw new Error(`There are no pods in '${deploymentName}' deployment.`)

  return podName
}

function createConnectingStates(commit, service) {
  for (const forward of service.forwards) {
    commit('SET', { port: forward.localPort, serviceId: service.id, state: connectionStates.CONNECTING })
  }
}

function clearStates(commit, service) {
  for (const forward of service.forwards) {
    commit('DELETE', forward.localPort)
  }
}

function validateThatRequiredPortsFree(state, service) {
  for (const forward of service.forwards) {
    if (state[forward.localPort]) {
      throw new Error(`Port ${forward.localPort} is busy.`)
    }
  }
}

let actions = {
  async createConnection({ commit, state, rootState }, service) {
    try {
      validateThatRequiredPortsFree(state, service)
      createConnectingStates(commit, service)

      const { kubeConfig, k8sPortForward } = prepareK8sToolsWithService(rootState, service)
      const podName = await getPodName(kubeConfig, service)
      const results = await Promise.all(service.forwards.map(forward =>
        startForward(commit, k8sPortForward, service, forward, podName)
      ))

      const success = !results.find(x => !x.success)
      if (!success) {
        for (const result of results) {
          killServer(commit, result.forward.localPort)
        }
      }

      return { success, results }
    } catch (error) {
      console.error(error)
      clearStates(commit, service)
      return { success: false, message: error.message }
    }
  },

  deleteConnection({ commit }, service) {
    for (const forward of service.forwards) {
      killServer(commit, forward.localPort)
    }
  }
}

if (isWebDemo) {
  actions = {
    createConnection({ commit }, service) {
      service.forwards.map(forward =>
        commit('SET', { port: forward.localPort, serviceId: service.id, state: connectionStates.CONNECTED })
      )
    },
    deleteConnection({ commit }, service) {
      service.forwards.map(forward =>
        commit('DELETE', forward.localPort)
      )
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
