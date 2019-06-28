import Vue from 'vue'
import Router from 'vue-router'

import analytics from '../analytics'

Vue.use(Router)

const router = new Router({
  // in Cypress 'hash' saves state between routes
  mode: process.env.NODE_ENV === 'test' ? 'hash' : 'history',
  routes: [
    {
      path: '/',
      component: require('@/components/Layout').default,
      children: [
        {
          name: 'Clusters',
          path: '',
          component: require('@/components/Clusters').default
        },
        {
          name: 'Cluster New',
          path: 'clusters/new',
          component: require('@/components/ClusterNew').default
        },
        {
          name: 'Cluster Import',
          path: 'clusters/import',
          component: require('@/components/ClusterImport').default
        },
        {
          name: 'Cluster Edit',
          path: 'clusters/:id/edit',
          component: require('@/components/ClusterEdit').default
        },
        {
          name: 'Service New',
          path: 'clusters/:clusterId/services/new',
          component: require('@/components/ServiceForm').default
        },
        {
          name: 'Service Edit',
          path: 'clusters/:clusterId/services/:id/edit',
          component: require('@/components/ServiceEdit').default
        },
        {
          name: 'Service Clone',
          path: 'clusters/:clusterId/services/:id/clone',
          component: require('@/components/ServiceClone').default
        }
      ]
    },
    {
      path: '*',
      redirect: '/'
    }
  ],
  scrollBehavior(to, from, savedPosition) {
    return { x: 0, y: 0 }
  }
})

router.afterEach((to, from) => {
  analytics.send('screenview', { cd: to.name })
})

export default router
