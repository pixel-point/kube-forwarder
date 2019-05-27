<template>
  <div class="clusters" :class="{ clusters_empty: clustersCount === 0 }">
    <template v-if="clustersCount">
      <Header>
        <SearchInput v-model="query" size="s" />
        <div class="space" />
        <Dropdown theme="primary">
          <template v-slot:trigger="triggerSlotProps">
            <Button :outline="!triggerSlotProps.opened" theme="primary" @click="triggerSlotProps.toggle">
              Add a cluster
              <IconArrowDropdown :to="triggerSlotProps.opened ? 'top' : 'bottom'" />
            </Button>
          </template>

          <ul class="popup__actions">
            <li><Action to="/clusters/new">FROM SCRATCH</Action></li>
            <li><Action to="/clusters/import">FROM IMPORT</Action></li>
          </ul>
        </Dropdown>
      </Header>

      <ClusterItem
        v-for="cluster in clustersToRender"
        :key="cluster.id"
        :cluster="cluster"
        :services="filteredServicesByClusterId[cluster.id]"
        :filtered="!!query"
      />

      <Alert v-if="!filteredClusterIds.length">Not found</Alert>
    </template>

    <template v-else>
      <Button size="l" theme="primary" to="/clusters/new">Add a cluster</Button>
    </template>
  </div>
</template>

<script>
import Button from './shared/Button'
import Header from './shared/Header'
import SearchInput from './shared/SearchInput'
import Dropdown from './shared/Dropdown'
import ClusterItem from './Clusters/ClusterItem'
import Alert from './shared/Alert'
import IconArrowDropdown from './shared/icons/IconArrowDropdown'
import Action from './shared/Action'

export default {
  name: 'Clusters',
  components: {
    Action,
    Button,
    Header,
    SearchInput,
    Dropdown,
    ClusterItem,
    Alert,
    IconArrowDropdown
  },
  data() {
    return {
      query: ''
    }
  },
  computed: {
    clustersToRender() {
      return this.filteredClusterIds
        .map(id => this.clustersById[id])
        .sort((a, b) => (
          this.compareNames(a.name.toLowerCase(), b.name.toLowerCase()) ||
          this.compareNames(a.name, b.name)
        ))
    },
    filteredServicesByClusterId() {
      const result = {}
      const services = Object.values(this.$store.state.Services.items)

      for (const service of services) {
        if (this.clustersById[service.clusterId] && (!this.query || service.name.includes(this.query))) {
          result[service.clusterId] = result[service.clusterId] || []
          result[service.clusterId].push(service)
        }
      }

      return result
    },
    filteredClusterIds() {
      if (this.query) return Object.keys(this.filteredServicesByClusterId)
      return Object.keys(this.clustersById)
    },
    clustersById() {
      return this.$store.state.Clusters.items
    },
    clustersCount() {
      return Object.keys(this.clustersById).length
    }
  },
  methods: {
    compareNames(a, b) {
      return a > b ? 1 : a < b ? -1 : 0
    }
  }
}
</script>

<style scoped lang="scss">
@import "../assets/styles/variables";

.clusters {
  padding: 20px;
  flex-grow: 1;

  .header {
    margin-bottom: 20px;

    .search-input {
      width: 158px;
    }
  }

  .cluster-item + .cluster-item {
    margin-top: 20px
  }
}

.clusters_empty {
  display: flex;
  justify-content: space-around;
  align-items: center;
}
</style>
