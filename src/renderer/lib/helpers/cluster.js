import { CoreV1Api, KubeConfig } from '@kubernetes/client-node'

import { k8nApiPrettyError } from './k8n-api-error'
import { showMessageBox } from './ui'

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
    const commandNotFoundMatch = e.message.match(/\s([a-zA-Z0-9]+: command not found)/)

    if (commandNotFoundMatch) {
      await showMessageBox(`${commandNotFoundMatch[1]}.`)
      error = e
    } else {
      error = k8nApiPrettyError(e)
    }
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
