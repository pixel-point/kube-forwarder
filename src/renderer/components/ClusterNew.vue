<template>
  <div class="page-cluster-new">
    <Header :back-path="backPath" />
    <ClusterForm />
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
