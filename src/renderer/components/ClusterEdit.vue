<template>
  <div class="page-cluster-edit">
    <Header :back-path="backPath" />
    <ClusterForm :cluster-id="clusterId" :initial-attributes="cluster" />
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
    },
    clusterId() {
      return this.$route.params.id
    },
    cluster() {
      return this.$store.state.Clusters.items[this.clusterId]
    }
  }
}

</script>

<style scoped lang="scss">
.page-cluster-edit {
  padding: 20px;

  .header {
    margin-bottom: 20px;
  }
}
</style>
