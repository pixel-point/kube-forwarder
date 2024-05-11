/* eslint-disable no-unused-vars */
import { v1 as uuidv1 } from 'uuid'

import { createToolset, commitIfValid } from '../helpers/validations'
import workloadTypes from '../../lib/constants/workload-types'
import { SET, DELETE } from '../helpers/mutations'

const state = {
  items: {
    // <serviceId>: <item>
  }
}

// TODO Renaming
//   "Service" module should be renamed to "Resource" to evade similarity with Service in Kubernetes
//   workloadType and workloadName could be renamed to kind and name.
//   Names should be changes due to new kind: 'Service'
//   I don't do renaming, since current configs at users and exports will be broken without proper migration
//   I could be good to accumulate more changes to do and do them in the v3 of config.
export const serviceSchema = {
  type: 'object',
  required: ['id', 'clusterId', 'namespace', 'workloadType', 'workloadName', 'forwards'],
  properties: {
    id: { type: 'string' },
    clusterId: { type: 'string' },
    alias: { type: 'string' },
    namespace: { type: 'string' },
    workloadType: { type: 'string', enum: workloadTypes },
    workloadName: { type: 'string' },
    forwards: {
      type: 'array',
      minItems: 1,
      items: {
        type: 'object',
        required: ['id', 'localPort', 'remotePort'],
        properties: {
          id: { type: 'string' },
          localPort: { type: 'integer', minimum: 0, maximum: 65535 },
          remotePort: { type: 'integer', minimum: 0, maximum: 65535 }
        }
      }
    },
    localAddress: { type: 'string' }
  }
}
const { validate, pick } = createToolset(serviceSchema)

const mutations = {
  CREATE: SET,
  UPDATE: SET,
  DELETE
}

function saveItem(commit, mutation, attributes) {
  const item = pick(attributes)
  return commitIfValid(commit, mutation, item, validate)
}

const actions = {
  createService({ commit }, attributes) {
    return saveItem(commit, 'CREATE', { ...attributes, id: uuidv1() })
  },

  updateService({ commit }, attributes) {
    return saveItem(commit, 'UPDATE', attributes)
  },

  deleteService({ commit }, id) {
    commit('DELETE', id)
  }
}

function cleanup(state, globalState) {
  // Remove services without a cluster
  for (const serviceId of Object.keys(state.items)) {
    const item = state.items[serviceId]

    if (!globalState.Clusters.items[item.clusterId]) {
      delete state.items[serviceId]
    }
  }
}

export default {
  namespaced: true,
  state,
  mutations,
  actions,
  cleanup
}
