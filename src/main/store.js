import ElectronStore from 'electron-store'

const schema = {
  notFirstLaunch: {
    type: 'boolean',
    default: false
  }
}

const electronStore = new ElectronStore({ schema })
const store = {}

for (const key of Object.keys(schema)) {
  Object.defineProperty(store, key, {
    get() {
      return electronStore.get(key)
    },
    set(value) {
      electronStore.set(key, value)
    }
  })
}

export default store
