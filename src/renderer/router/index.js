import Vue from 'vue'
import Router from 'vue-router'

Vue.use(Router)

export default new Router({
  routes: [
    {
      path: '/',
      component: require('@/components/Layout').default,
      children: [
        {
          path: '',
          component: require('@/components/Clusters').default
        },
        {
          path: 'clusters/new',
          component: require('@/components/ClusterNew').default
        },
        {
          path: 'clusters/import',
          component: require('@/components/ClusterImport').default
        },
        {
          path: 'clusters/:id/edit',
          component: require('@/components/ClusterEdit').default
        },
        {
          path: 'clusters/:clusterId/services/new',
          component: require('@/components/ServiceForm').default
        },
        {
          path: 'clusters/:clusterId/services/:id/edit',
          component: require('@/components/ServiceEdit').default
        },
        {
          path: 'clusters/:clusterId/services/:id/clone',
          component: require('@/components/ServiceClone').default
        }
      ]
    },
    {
      path: '*',
      redirect: '/'
    }
  ]
})
