import uuidv1 from 'uuid/v1'

import { createToolset, commitIfValid } from '../helpers/validations'
import { SET, DELETE } from '../helpers/mutations'
import * as configStoringMethods from '../../lib/constants/config-storing-methods'

const state = {
  items: {}
}

export const clusterSchema = {
  type: 'object',
  required: ['id', 'name', 'config'],
  additionalProperties: false,
  properties: {
    id: { type: 'string' },
    name: { type: 'string' },
    folded: { type: 'boolean' },
    config: {
      type: 'object',
      required: ['storingMethod'],
      additionalProperties: false,
      properties: {
        storingMethod: { type: 'string', enum: configStoringMethods.default },
        path: { type: 'string' },
        content: { type: 'string' },
        currentContext: { type: 'string' }
      },
      oneOf: [
        {
          properties: { storingMethod: { const: configStoringMethods.PATH } },
          required: ['path', 'currentContext']
        },
        {
          properties: { storingMethod: { const: configStoringMethods.CONTENT } },
          required: ['content']
        }
      ]
    }
  }
}
const { validate, pick } = createToolset(clusterSchema)

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
  createCluster({ commit }, attributes) {
    return saveItem(commit, 'CREATE', { ...attributes, id: uuidv1() })
  },

  updateCluster({ state, commit }, attributes) {
    if (!attributes.id) throw new Error('attributes.id must present')
    const currentItem = state.items[attributes.id]

    return saveItem(commit, 'UPDATE', { ...currentItem, ...attributes })
  },

  deleteCluster({ commit }, id) {
    commit('DELETE', id)
  }
}

export default {
  namespaced: true,
  state,
  mutations,
  actions
}
