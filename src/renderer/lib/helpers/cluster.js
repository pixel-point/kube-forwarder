import { Core_v1Api, KubeConfig } from '@kubernetes/client-node' // eslint-disable-line camelcase

import { k8nApiPrettyError } from './k8n-api-error'

export async function checkConnection(kubeConfig, context = null) {
  if (!kubeConfig || typeof kubeConfig.makeApiClient !== 'function') return

  let error = null
  const currentContext = kubeConfig.getCurrentContext()

  if (context) {
    kubeConfig.setCurrentContext(context)
  }

  try {
    const api = kubeConfig.makeApiClient(Core_v1Api)
    await api.listNode()
  } catch (e) {
    error = k8nApiPrettyError(e)
  }

  kubeConfig.setCurrentContext(currentContext)

  return error
}

// You must catch errors manually
export function buildApiClient(cluster, api) {
  const kubeConfig = new KubeConfig()
  kubeConfig.loadFromString(cluster.config)
  return kubeConfig.makeApiClient(api)
}
