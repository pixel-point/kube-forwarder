<template>
  <div :class="className">
    <div class="cluster-item__header" @click="handleUnfold">
      <div class="cluster-item__title">{{ cluster.name }}</div>
      <div class="space" />

      <IconArrowDropdown v-if="cluster.folded" />
      <Dropdown v-else :popup-props="{ offsetHorizontal: 1 }">
        <template v-slot:trigger="triggerSlotProps">
          <Button class="cluster-item__action-more" layout="text" @click="triggerSlotProps.toggle">
            <IconDotes />
          </Button>
        </template>

        <ul class="popup__actions">
          <li><Action :to="createServicePath">Add a Resource</Action></li>
          <li><Action :to="editPath">Edit</Action></li>
          <li><Action @click="exportCluster">Export</Action></li>
          <li><Action @click="startAll">Start all</Action></li>
          <li><Action @click="stopAll">Stop all</Action></li>
          <li><Action @click="handleFold">Collapse</Action></li>
          <li><Action theme="danger" @click="handleDelete">Delete</Action></li>
        </ul>
      </Dropdown>
    </div>

    <div v-if="!cluster.folded" class="cluster-item__services">
      <template v-if="computedServices.length">
        <ServiceItem v-for="service in computedServices" :key="service.id" :service="service" />
      </template>

      <div v-else class="cluster-item__empty">
        <Button theme="primary" :to="createServicePath">Add a resource</Button>
      </div>
    </div>
  </div>
</template>

<script>
import { mapActions } from 'vuex'
import { exportCluster, saveObjectToJsonFile } from '../../lib/export'
import { showSaveDialog, showErrorBox, showMessageBox, showConfirmBox } from '../../lib/helpers/ui'
import { CURRENT_STATE_VERSION } from '../../store'
import * as Sentry from '@sentry/electron'

import Dropdown from '../shared/Dropdown'
import IconDotes from '../shared/icons/IconDotes'
import ServiceItem from './ServiceItem'
import Action from '../shared/Action'
import IconArrowDropdown from '../shared/icons/IconArrowDropdown'
import Button from '../shared/Button'

export default {
  components: {
    IconArrowDropdown,
    Action,
    Dropdown,
    IconDotes,
    ServiceItem,
    Button
  },
  props: {
    cluster: { type: Object, required: true },
    services: { type: Array, default: null },
    filtered: { type: Boolean, default: false }
  },
  computed: {
    className() {
      return { 'cluster-item': true, 'cluster-item_folded': this.cluster.folded }
    },
    computedServices() {
      if (this.services) return this.services

      return Object.values(this.$store.state.Services.items).filter(service => service.clusterId === this.cluster.id)
    },
    createServicePath() {
      return `clusters/${this.cluster.id}/services/new`
    },
    editPath() {
      return `clusters/${this.cluster.id}/edit`
    }
  },
  methods: {
    ...mapActions('Clusters', ['updateCluster']),
    async handleDelete() {
      const confirm = await showConfirmBox(`Are you sure to delete "${this.cluster.name}" cluster`)
      if (confirm) {
        this.$store.dispatch('Clusters/deleteCluster', this.cluster.id)
      }
    },
    handleFold(e) {
      this.setFolded(true)
      e.stopPropagation()
    },
    handleUnfold() {
      if (this.cluster.folded) this.setFolded(false)
    },
    async setFolded(folded) {
      const { id } = this.cluster
      const result = await this.updateCluster({ id, folded })

      if (result.errors) alert(JSON.stringify(result.errors))
    },
    async exportCluster() {
      const [buttonIndex, checkboxChecked] = await showMessageBox(`Export cluster: ${this.cluster.name}`, {
        checkboxLabel: 'Include cluster config',
        buttons: ['Export', 'Cancel']
      })

      if (buttonIndex === 0) {
        let objectToExport

        try {
          objectToExport = await exportCluster(this.$store.state, this.cluster.id, {
            includeConfig: checkboxChecked
          })
        } catch (error) {
          Sentry.captureException(error)
          showErrorBox(error.message)
          return
        }

        const defaultName = `cluster-${this.cluster.name}.kpf-export.v${CURRENT_STATE_VERSION}.json`
        const filename = await showSaveDialog({ defaultName })

        if (filename) {
          try {
            saveObjectToJsonFile(objectToExport, filename)
          } catch (error) {
            console.error(error)
            showErrorBox(`Sorry, an error occurred while saving the file. Raw error message: ${error.message}`)
          }
        }
      }
    },
    startAll() {
      this.services.forEach(service => {
        this.$store.dispatch('Connections/createConnection', service)
      })
    },
    stopAll() {
      this.services.forEach(service => {
        this.$store.dispatch('Connections/deleteConnection', service)
      })
    }
  }
}

</script>

<style lang="scss">
@import "../../assets/styles/variables";

.cluster-item {
  border: $border;
  border-radius: $border-radius;
  box-shadow: 0 5px 10px $border-color;
}

.cluster-item_folded {
  cursor: pointer;
  box-shadow: none;

  .cluster-item__header {
    padding-right: 10px;
    border: none;
    height: 34px;
  }
}

.cluster-item__header {
  background: $bg-secondary;
  font-size: $font-size-big;
  font-weight: 500;
  color: $color-text-tertiary;
  height: 35px;
  padding: 0 0 0 15px;
  display: flex;
  align-items: center;
  border-bottom: $border;

  .icon_arrow-dropdown {
    cursor: pointer;
    color: $color-text-quaternary;
  }
}

.cluster-item__action-more {
  padding: 0;
  width: 24px;
}

.cluster-item__empty {
  color: $color-text-tertiary;
  padding: 21px;
  text-align: center;
}

.cluster-item__services {
  .service-item + .service-item {
    border-top: $border;
  }
}
</style>
