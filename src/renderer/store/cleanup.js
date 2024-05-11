/* eslint-disable no-unused-vars */
import { difference } from 'lodash'
import modules from './modules'

export default function(state, persistedModuleNames, persistedKeys) {
  for (const moduleName of persistedModuleNames) {
    const module = modules[moduleName]
    module.cleanup && module.cleanup(state[moduleName], state)
  }

  const keysToClear = difference(Object.keys(state), persistedKeys)

  for (const key of keysToClear) {
    if (modules[key]) {
      state[key] = modules[key].state
    } else {
      delete state[key]
    }
  }
}
