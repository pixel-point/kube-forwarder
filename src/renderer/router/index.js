import Vue from 'vue'
import Router from 'vue-router'
import * as Sentry from '@sentry/electron'

import store from '../store'
import analytics from '../analytics'

Vue.use(Router)

const router = new Router({
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
          name: 'Cluster Add',
          path: 'clusters/add',
          component: require('@/components/ClusterAdd').default
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
          component: require('@/components/ServiceNew').default
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
  Sentry.addBreadcrumb({ message: `Navigated from '${from.name}' to '${to.name}'` })

  // todo Move it in the right place.
  if (from.name === 'Cluster Add') {
    delete store.state.manualClusterConfig
  }
})

export default router
