import { createToolset } from '../helpers/validations'
import { plain } from '../../lib/plain'
import * as connectionStates from '../../lib/constants/connection-states'

// The forwarding engine (k8s client + local TCP servers) lives in the MAIN
// process (see src/main/k8s.js). This module only holds UI state, keyed by
// `address:port`. Mutations are driven both by local actions and by events the
// main process pushes over IPC (wired up in store/index.js).

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

const state = {}

const mutations = {
  SET(state, payloadedItem) {
    const item = { flags: { http: false }, ...payloadedItem }

    const valid = validate(item)
    if (valid) state[[item.address, item.port].join(':')] = item
    else throw new Error(JSON.stringify(validate.errors))
  },

  SET_FLAG(state, { address, port, flagName, flagValue }) {
    if (!port) throw new Error('port must be present')
    if (!flagName) throw new Error('flagName must be present')
    const key = [address, port].join(':')
    if (state[key]) {
      state[key].flags[flagName] = flagValue
    }
  },

  DELETE(state, { address, port }) {
    if (!port) throw new Error('port must present')
    delete state[[address, port].join(':')]
  }
}

const actions = {
  createConnection({ rootState }, service) {
    const cluster = rootState.Clusters.items[service.clusterId]
    return window.api.connections.create(plain({ service, cluster }))
  },

  deleteConnection(_context, service) {
    return window.api.connections.delete(plain(service))
  }
}

export default {
  persisted: false,
  namespaced: true,
  state,
  mutations,
  actions
}
