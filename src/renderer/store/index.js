import { createStore } from 'vuex'
import deepmerge from 'deepmerge'

import modules from './modules'
import migrate from './migrate'
import cleanup from './cleanup'
import isVersion1 from './helpers/is-version-1'

export const CURRENT_STATE_VERSION = 2

const persistedModuleNames = Object.keys(modules).filter(name => modules[name].persisted !== false)
const persistedKeys = ['version', ...persistedModuleNames]

const store = createStore({
  state: {
    version: CURRENT_STATE_VERSION
  },
  modules,
  strict: import.meta.env.DEV,
  mutations: {
    CLEANUP(state) {
      cleanup(state, persistedModuleNames, persistedKeys)
    },
    MIGRATE(state) {
      migrate(state)
    },
    SET_VERSION(state, { version }) {
      state.version = version
    }
  }
})

// Hydrate persisted state from disk (electron-store in main, read synchronously
// so store setup stays synchronous — matches the old vuex-electron behavior).
try {
  const persisted = window.api.store.getSync()
  if (persisted) {
    store.replaceState(deepmerge(store.state, persisted, { arrayMerge: (_dst, src) => src }))
  }
} catch (e) {
  console.error('Failed to load persisted state', e)
}

// Persist filtered state to disk on every mutation.
store.subscribe((_mutation, state) => {
  const filtered = {}
  for (const key of persistedKeys) filtered[key] = state[key]
  try {
    // JSON round-trip strips Vue's reactive Proxy to a plain object; a Proxy
    // isn't structured-cloneable and IPC would throw "could not be cloned".
    window.api.store.set(JSON.parse(JSON.stringify(filtered)))
  } catch (e) {
    console.error('Failed to persist state', e)
  }
})

if (isVersion1(store.state)) {
  store.commit('SET_VERSION', { version: 1 })
}

store.commit('MIGRATE')
store.commit('CLEANUP')

// Port-forward state changes are computed in the main process and pushed here.
try {
  window.api.connections.onEvent(({ type, payload }) => {
    store.commit(`Connections/${type}`, payload)
  })
} catch (e) {
  console.error('Failed to subscribe to connection events', e)
}

export default store
