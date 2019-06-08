<template>
  <div class="clusters" :class="{ clusters_empty: clustersCount === 0 }">
    <template v-if="clustersCount">
      <Header>
        <SearchInput v-model="query" size="s" />
        <div class="space" />
        <div class="clusters__header-actions">
          <Dropdown :popup-props="{ align: 'both' }" class="clusters__add-cluster-dropdown">
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

          <Dropdown class="clusters__more-dropdown">
            <template v-slot:trigger="triggerSlotProps">
              <Button outline @click="triggerSlotProps.toggle">
                <IconDotes />
              </Button>
            </template>

            <template v-slot="slotProps">
              <ul class="popup__actions">
                <li v-if="isEveryClusterFolded"><Action @click="unfoldAll(slotProps.close)">Unfold all clusters</Action></li>
                <li v-else><Action @click="foldAll(slotProps.close)">Fold all clusters</Action></li>
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

      <Alert v-if="!filteredClusterIds.length">Not found</Alert>
    </template>

    <template v-else>
      <h1>Add a cluster</h1>

      <div class="clusters__controls">
        <Button theme="secondary" size="l" to="/clusters/new">FROM SCRATCH</Button>
        <div class="cluster__control-or">OR</div>
        <Button theme="primary" size="l" to="/clusters/import">FROM IMPORT</Button>
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
import Alert from './shared/Alert'
import IconArrowDropdown from './shared/icons/IconArrowDropdown'
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
    Alert,
    IconArrowDropdown,
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
  padding: 20px;
  flex-grow: 1;

  .header {
    margin-bottom: 20px;

    .search-input .base-input {
      width: 191px;
      transition: width linear 0.2s;

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
    margin: 58px 0 60px;
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
    width: 22px;
    padding: 0;
  }
}

.cluster__control-or {
  line-height: 20px;
  margin: 0 30px;
  font-size: 17px;
  color: $color-text-secondary;
}
</style>
