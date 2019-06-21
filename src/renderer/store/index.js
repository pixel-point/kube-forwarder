import Vue from 'vue'
import Vuex from 'vuex'

import { createPersistedState } from 'vuex-electron'

import modules from './modules'
import { isWebDemo } from '../lib/environment'
import { importCluster } from '../lib/export'

Vue.use(Vuex)

const persistedModuleNames = []
const noPersistedModuleNames = []

Object.keys(modules).forEach(moduleName => {
  const isPersisted = modules[moduleName].persisted !== false;
  (isPersisted ? persistedModuleNames : noPersistedModuleNames).push(moduleName)
})

const store = new Vuex.Store({
  modules,
  plugins: [
    !process.env.IS_WEB && createPersistedState({ paths: persistedModuleNames })
  ].filter(Boolean),
  strict: process.env.NODE_ENV !== 'production',
  mutations: {
    CLEANUP(state) {
      for (const moduleName of noPersistedModuleNames) {
        state[moduleName] = modules[moduleName].state
      }
    }
  }
})

// Fake demo data
if (isWebDemo) {
  const [cluster, services] = importCluster(require('../../../static/cluster-Pixel Point.kpf-export'))

  store.dispatch('Clusters/createCluster', cluster).then(async (result) => {
    const { item } = result
    const results = await Promise.all(services.map(service => (
      store.dispatch('Services/createService', { ...service, clusterId: item.id })
    )))

    if (results[1]) {
      store.dispatch('Connections/createConnection', results[1].item)
    }
  })
}

store.commit('CLEANUP')

export default store
