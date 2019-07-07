<template>
  <div :class="{ page: true, clusters: true, clusters_empty: clustersCount === 0, 'clusters_not-found': !filteredClusterIds.length }">
    <template v-if="clustersCount">
      <Header>
        <SearchInput v-model="query" size="s" />
        <div class="space" />
        <div class="clusters__header-actions">
          <Button layout="outline" theme="primary" to="/clusters/add">Add a cluster</Button>

          <Dropdown class="clusters__more-dropdown">
            <template v-slot:trigger="triggerSlotProps">
              <Button layout="outline" @click="triggerSlotProps.toggle">
                <IconDotes />
              </Button>
            </template>

            <template v-slot="slotProps">
              <ul class="popup__actions">
                <li v-if="isEveryClusterFolded">
                  <Action @click="unfoldAll(slotProps.close)">Show All Clusters</Action>
                </li>
                <li v-else><Action @click="foldAll(slotProps.close)">Collapse All Clusters</Action></li>
              </ul>
            </template>
          </Dropdown>
        </div>
      </Header>

      <ClusterItem
        v-for="cluster in clustersToRender"
        :key="cluster.id"
        :cluster="cluster"
        :services="filteredServicesByClusterId[cluster.id]"
        :filtered="!!query"
      />

      <div v-if="!filteredClusterIds.length" class="text_theme_secondary">Nothing found...</div>
    </template>

    <template v-else>
      <h1>Add your <span class="text_theme_gradient">first cluster</span> before<br/>you start forward services</h1>

      <div class="clusters__controls">
        <Button theme="primary" size="l" to="/clusters/add">ADD A CLUSTER</Button>
      </div>
    </template>
  </div>
</template>

<script>
import { mapActions } from 'vuex'

import Button from './shared/Button'
import Header from './shared/Header'
import SearchInput from './shared/SearchInput'
import Dropdown from './shared/Dropdown'
import ClusterItem from './Clusters/ClusterItem'
import IconDotes from './shared/icons/IconDotes'
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
    IconDotes
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
        if (
          this.clustersById[service.clusterId] && // cluster present
          (
            // no query or service matches to query
            !this.query ||
            (
              (service.alias && service.alias.includes(this.query)) ||
              service.workloadName.includes(this.query)
            )
          )
        ) {
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
    },
    isEveryClusterFolded() {
      return this.clustersToRender.every(cluster => cluster.folded)
    }
  },
  methods: {
    ...mapActions('Clusters', ['updateCluster']),
    compareNames(a, b) {
      return a > b ? 1 : a < b ? -1 : 0
    },
    foldAll(callback) {
      this.setFoldedToAll(true)
      callback()
    },
    unfoldAll(callback) {
      this.setFoldedToAll(false)
      callback()
    },
    setFoldedToAll(folded) {
      this.clustersToRender.forEach(cluster => {
        this.updateCluster({ id: cluster.id, folded })
      })
    }
  }
}
</script>

<style lang="scss">
@import "../assets/styles/variables";

.clusters {
  flex-grow: 1;

  .header {
    .search-input .base-input {
      width: 191px;
      transition: width $hover-transition-speed, border-color $hover-transition-speed;

      &:focus {
        width: 320px;
      }
    }
  }

  .cluster-item + .cluster-item {
    margin-top: 20px
  }
}

.clusters_empty {
  h1 {
    font-size: 30px;
    text-align: center;
    line-height: 36px;
    margin: 63px 0 40px;
    font-weight: normal;
  }

  &:after {
    content: "";
    background: url('../assets/images/pattern@2x.png') no-repeat;
    background-size: contain;
    opacity: 0.75;
    height: 180px;
    left: 0;
    bottom: 0;
    right: 0;
    position: absolute;
  }
}

.clusters_not-found {
  .header {
    z-index: 10
  }

  .text_theme_secondary {
    position: fixed;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
    z-index: 0;
    top: 0;
    bottom: 0;
    left: 0;
    right: 0;
  }
}

.clusters__controls {
  display: flex;
  align-items: center;
  justify-content: center;

  .button {
    width: 220px;
  }
}

.clusters__header-actions {
  font-size: 0;

  .popup__actions {
    font-size: $font-size-base;
  }
}

.clusters__add-cluster-dropdown {
  .popup__actions {
    font-size: $font-size-small;
  }
}

.clusters__more-dropdown {
  margin-left: 13px;

  .button {
    width: 26px;
    padding: 0;
  }
}

.cluster__control-or {
  line-height: 20px;
  margin: 0 30px;
  font-size: 17px;
  color: var(--color-text-secondary);
}
</style>
