import { createRouter, createWebHashHistory } from 'vue-router'

import store from '../store'
import analytics from '../analytics'

import Layout from '@/components/Layout'
import Clusters from '@/components/Clusters'
import ClusterNew from '@/components/ClusterNew'
import ClusterAdd from '@/components/ClusterAdd'
import ClusterImport from '@/components/ClusterImport'
import ClusterEdit from '@/components/ClusterEdit'
import ServiceNew from '@/components/ServiceNew'
import ServiceEdit from '@/components/ServiceEdit'
import ServiceClone from '@/components/ServiceClone'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      component: Layout,
      children: [
        { name: 'Clusters', path: '', component: Clusters },
        { name: 'Cluster New', path: 'clusters/new', component: ClusterNew },
        { name: 'Cluster Add', path: 'clusters/add', component: ClusterAdd },
        { name: 'Cluster Import', path: 'clusters/import', component: ClusterImport },
        { name: 'Cluster Edit', path: 'clusters/:id/edit', component: ClusterEdit },
        { name: 'Service New', path: 'clusters/:clusterId/services/new', component: ServiceNew },
        { name: 'Service Edit', path: 'clusters/:clusterId/services/:id/edit', component: ServiceEdit },
        { name: 'Service Clone', path: 'clusters/:clusterId/services/:id/clone', component: ServiceClone }
      ]
    },
    {
      path: '/:pathMatch(.*)*',
      redirect: '/'
    }
  ],
  scrollBehavior() {
    return { left: 0, top: 0 }
  }
})

router.afterEach((to, from) => {
  analytics.send('screenview', { cd: to.name })

  // todo Move it in the right place.
  if (from.name === 'Cluster Add') {
    delete store.state.manualClusterConfig
  }
})

export default router
