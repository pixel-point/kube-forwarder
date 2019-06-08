const indexes = {}

function createGenerator(name, fn) {
  indexes[name] = 0
  return () => {
    indexes[name] += 1
    return fn(indexes[name])
  }
}

const generateClusterId = createGenerator('clusterId', (i) => `cluster-id-${i}`)
const generateServiceId = createGenerator('serviceId', (i) => `service-id-${i}`)
const generateForwardId = createGenerator('forwardId', (i) => `forward-id-${i}`)

export function buildCluster(overrides = {}) {
  return {
    id: generateClusterId(),
    name: 'cluster-name',
    config: '< Cluster-config >',
    folded: false,
    ...overrides
  }
}

export function buildService(overrides = {}, bCluster = buildCluster) {
  const clusterId = overrides.clusterId || bCluster().id

  return {
    id: generateServiceId(),
    clusterId,
    alias: 'service-alias',
    namespace: 'service-namespace',
    workloadType: 'pod',
    workloadName: 'workloadName',
    forwards: [
      { id: generateForwardId(), localPort: 3000, remotePort: 4000 }
    ],
    ...overrides
  }
}

export class StateBuilder {
  constructor() {
    this.state = { Clusters: { items: {} }, Services: { items: {} } }
  }

  getState() {
    return this.state
  }

  createCluster(overrides) {
    const item = buildCluster(overrides)
    this.state.Clusters.items[item.id] = item
    return item
  }

  createService(overrides) {
    const item = buildService(overrides, this.createCluster.bind(this))
    this.state.Services.items[item.id] = item
    return item
  }
}
