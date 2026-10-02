/**
 * Imports all vuex modules in this folder in one shot (Vite equivalent of the
 * old webpack `require.context`). There should not be any reason to edit this file.
 */
const files = import.meta.glob('./*.js', { eager: true })
const modules = {}

for (const key of Object.keys(files)) {
  if (key === './index.js') continue
  modules[key.replace(/(\.\/|\.js)/g, '')] = files[key].default
}

export default modules
