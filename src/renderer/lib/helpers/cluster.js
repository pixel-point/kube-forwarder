import { CoreV1Api, KubeConfig } from '@kubernetes/client-node'

import { k8nApiPrettyError } from './k8n-api-error'
import * as configStoringMethods from '../constants/config-storing-methods'

// You must catch errors manually
export function buildKubeConfig(clusterConfig) {
  const kubeConfig = new KubeConfig()

  if (clusterConfig.storingMethod === configStoringMethods.PATH) {
    console.log('pa')
    kubeConfig.loadFromFile(clusterConfig.path)
  } else if (clusterConfig.storingMethod === configStoringMethods.CONTENT) {
    console.log('co')
    kubeConfig.loadFromString(clusterConfig.content)
  } else {
    throw new Error(`storingMethod "${clusterConfig.storingMethod}" is invalid.`)
  }

  return kubeConfig
}

export async function checkConnection(kubeConfig, context = null) {
  if (!kubeConfig || typeof kubeConfig.makeApiClient !== 'function') return

  let error = null
  const currentContext = kubeConfig.getCurrentContext()

  if (context) {
    kubeConfig.setCurrentContext(context)
  }

  try {
    const api = kubeConfig.makeApiClient(CoreV1Api)
    await api.listNode()
  } catch (e) {
    error = k8nApiPrettyError(e)
  }

  kubeConfig.setCurrentContext(currentContext)

  return error
}

// You must catch errors manually
export function buildApiClient(cluster, api) {
  const kubeConfig = buildKubeConfig(cluster.config)
  return kubeConfig.makeApiClient(api)
}
