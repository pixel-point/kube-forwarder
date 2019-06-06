<template>
  <div class="page-cluster-new">
    <Header :back-path="backPath" />
    <ClusterForm @success="handleSuccess" />
  </div>
</template>

<script>
import ClusterForm from './shared/cluster/ClusterForm'
import Header from './shared/Header'

export default {
  components: {
    ClusterForm,
    Header
  },
  computed: {
    backPath() {
      return '/'
    }
  },
  methods: {
    async handleSuccess(cluster) {
      this.$router.push(`/clusters/${cluster.id}/services/new`)
    }
  }
}

</script>

<style scoped lang="scss">
.page-cluster-new {
  padding: 20px;

  .header {
    margin-bottom: 20px;
  }
}
</style>
