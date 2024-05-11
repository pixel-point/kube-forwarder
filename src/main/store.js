import ElectronStore from 'electron-store'

const schema = {
  notFirstLaunch: {
    type: 'boolean',
    default: false
  }
}

const electronStore = new ElectronStore({ schema })
const store = {}

Object.keys(schema).forEach((key) => {
  Object.defineProperty(store, key, {
    get() {
      return electronStore.get(key)
    },
    set(value) {
      electronStore.set(key, value)
    }
  })
})

export default store
