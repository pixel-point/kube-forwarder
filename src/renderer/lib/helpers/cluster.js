// Thin renderer-side wrappers over the main-process k8s handlers (see
// src/main/k8s.js). KubeConfig and the k8s client are Node-only and live in main.
import { plain } from '../plain'

// Returns a plain error object `{ message, originMessage }` or null.
export function checkConnection(clusterConfig, context = null) {
  return window.api.cluster.check(plain(clusterConfig), context)
}

// Returns `{ contexts, currentContext }` or `{ error }`.
export function contextsFromString(content) {
  return window.api.cluster.contextsFromString(content)
}

export function contextsFromFile(filePath) {
  return window.api.cluster.contextsFromFile(filePath)
}
