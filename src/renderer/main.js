import { createApp } from 'vue'

import configureSentry from './configure-sentry'

import App from './App'
import router from './router'
import store from './store'

const app = createApp(App)
configureSentry({ app })
app.use(store).use(router).mount('#app')

if (import.meta.env.DEV) {
  window.app = app
}
