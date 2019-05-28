import Vue from 'vue'
import Vuex from 'vuex'

import { createPersistedState } from 'vuex-electron'

import modules from './modules'

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
    // createSharedMutations()
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

store.commit('CLEANUP')

export default store
