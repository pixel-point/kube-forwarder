import { Core_v1Api } from '@kubernetes/client-node' // eslint-disable-line camelcase

import { showMessageBox } from './ui'
import { k8nApiPrettyError } from './k8n-api-error'

export async function checkConnection(kubeConfig, context = null) {
  if (!kubeConfig || typeof kubeConfig.makeApiClient !== 'function') return

  const currentContext = kubeConfig.getCurrentContext()

  if (context) {
    kubeConfig.setCurrentContext(context)
  }

  try {
    const api = kubeConfig.makeApiClient(Core_v1Api)
    await api.listNode()
    await showMessageBox('Connection successful')
  } catch (e) {
    const error = k8nApiPrettyError(e)
    await showMessageBox('Connection failed', {
      details: `${error.message}${error.details ? `\n${error.details}` : ''}`
    })
  }

  kubeConfig.setCurrentContext(currentContext)
}
