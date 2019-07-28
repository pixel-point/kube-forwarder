<template>
  <div class="page-cluster-import">
    <Loader v-if="importing" />
    <template v-else>
      <Header back-path="/" />
      <ClusterForm :initial-attributes="cluster" :message="formMessage" @success="handleSuccess" />
    </template>
  </div>
</template>

<script>
import { showOpenDialog, showErrorBox } from '../lib/helpers/ui'
import { importCluster, readObjectFromJsonFile } from '../lib/export'

import ClusterForm from './shared/cluster/ClusterForm'
import Loader from './shared/Loader'
import Header from './shared/Header'

export default {
  components: { ClusterForm, Loader, Header },
  data() {
    return {
      importing: true,
      cluster: null,
      services: null
    }
  },
  computed: {
    formMessage() {
      if (this.services && this.services.length) {
        return `The cluster will be imported with ${this.services.length} resources`
      }

      return null
    },
    backPath() {
      return this.$route.query.back || '/'
    }
  },
  async mounted() {
    const filePaths = await showOpenDialog({ properties: ['openFile'], filters: [{ extensions: ['.json'] }] })
    if (filePaths) {
      try {
        const object = await readObjectFromJsonFile(filePaths[0])
        const [cluster, services] = importCluster(object)

        this.cluster = cluster
        this.services = services
        this.importing = false
        return
      } catch (error) {
        showErrorBox(error.message)
      }
    }
    this.$router.push(this.backPath)
  },
  methods: {
    async handleSuccess(cluster) {
      if (this.services && this.services.length) {
        await Promise.all(this.services.map(service => {
          return this.$store.dispatch('Services/createService', {
            ...service,
            clusterId: cluster.id
          })
        }))
      }

      this.$router.push('/')
    }
  }
}
</script>

<style lang="scss">
.page-cluster-import {
  padding: 20px;
  flex-grow: 1;
  display: flex;
  flex-direction: column;

  > .loader {
    flex-grow: 1;
  }

  > .header {
    margin-bottom: 20px;
  }
}
</style>
