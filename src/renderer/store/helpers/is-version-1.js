export default function(state) {
  const cluster = Object.values(state.Clusters.items)[0]
  if (cluster && (typeof cluster.config === 'string' || typeof cluster.config === 'undefined')) {
    return true
  }
}
