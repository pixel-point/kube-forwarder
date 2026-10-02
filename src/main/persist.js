// Renderer vuex state persisted to disk (replaces the abandoned vuex-electron).
import ElectronStore from 'electron-store'

const store = new ElectronStore({ name: 'vuex' })

export function getState() {
  return store.get('state') ?? null
}

export function setState(state) {
  store.set('state', state)
}
