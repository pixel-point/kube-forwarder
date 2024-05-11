import Vue from 'vue'

/* eslint-disable import/first */
import fixPath from './lib/fix-path'
fixPath()

import App from './App'
import router from './router'
import store from './store'
import './lib/k8s-shelljs-replace-patch'
/* eslint-enable import/first */

if (!process.env.IS_WEB) Vue.use(require('vue-electron'))
Vue.config.productionTip = false

const vue = new Vue({
  components: { App },
  router,
  store,
  template: '<App/>'
}).$mount('#app')

if (process.env.NODE_ENV !== 'production') {
  window.Vue = Vue
  window.vue = vue
}
