<template>
  <div :class="className">
    <div class="cluster-item__header">
      <div class="cluster-item__title">{{ cluster.name }}</div>
      <div class="space" />

      <IconArrowDropdown v-if="cluster.hidden" @click.native="show" />
      <Dropdown v-else>
        <template v-slot:trigger="triggerSlotProps">
          <IconDotes @click.native="triggerSlotProps.toggle" />
        </template>

        <ul class="popup__actions">
          <li><Action :to="createServicePath">Add a Service</Action></li>
          <li><Action :to="editPath">Edit</Action></li>
          <li><Action @click="exportCluster">Export</Action></li>
          <li><Action @click="hide">Hide</Action></li>
          <li><Action theme="danger" @click="handleDelete">Delete</Action></li>
        </ul>
      </Dropdown>
    </div>

    <div v-if="!cluster.hidden" class="cluster-item__services">
      <template v-if="computedServices.length">
        <ServiceItem v-for="service in computedServices" :key="service.id" :service="service" />
      </template>

      <div v-else class="cluster-item__empty">Services list is empty</div>
    </div>
  </div>
</template>

<script>
import { mapActions } from 'vuex'
import { exportCluster, saveObjectToJsonFile } from '../../lib/export'
import { showSaveDialog, showErrorBox, showMessageBox, showConfirmBox } from '../../lib/helpers/ui'

import Dropdown from '../shared/Dropdown'
import IconDotes from '../shared/icons/IconDotes'
import ServiceItem from './ServiceItem'
import Action from '../shared/Action'
import IconArrowDropdown from '../shared/icons/IconArrowDropdown'

export default {
  components: {
    IconArrowDropdown,
    Action,
    Dropdown,
    IconDotes,
    ServiceItem
  },
  props: {
    cluster: { type: Object, required: true },
    services: { type: Array, default: null },
    filtered: { type: Boolean, default: false }
  },
  computed: {
    className() {
      return { 'cluster-item': true, 'cluster-item_hidden': this.cluster.hidden }
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
    },
    forwardAllConflicts() {
      if (!this.services) return null

      const serviceNamesByPort = {}
      for (const service of this.services) {
        for (const forward of service.forwards) {
          serviceNamesByPort[forward.localPort] = serviceNamesByPort[forward.localPort] || []
          serviceNamesByPort[forward.localPort].push(service.name)
        }
      }

      const conflicts = []
      for (const localPort of Object.keys(serviceNamesByPort)) {
        const serviceNames = serviceNamesByPort[localPort]
        if (serviceNames.length > 1) {
          conflicts.push({ services: serviceNames, port: localPort })
        }
      }

      return conflicts.length ? conflicts : null
    },
    canForwardAll() {
      return !this.filtered && !this.forwardAllConflicts && false // since not implemented
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
    forwardAll() {
      if (this.filtered) {
        alert('Please clear the search field to forward all services')
      }

      if (this.forwardAllConflicts) {
        const excuses = 'Sorry, you can\'t forward all services.'
        const conflict = this.forwardAllConflicts[0]

        alert(`${excuses} Services "${conflict.services.join('", "')}" uses the same port ${conflict.port}`)
      }

      // TODO
      alert('Not implemented yet.')
    },
    hide() {
      this.setHidden(true)
    },
    show() {
      this.setHidden(false)
    },
    async setHidden(hidden) {
      const { id } = this.cluster
      const result = await this.updateCluster({ id, hidden })

      if (result.errors) alert(JSON.stringify(result.errors))
    },
    async exportCluster() {
      const [buttonIndex, checkboxChecked] = await showMessageBox(`Export cluster: ${this.cluster.name}`, {
        checkboxLabel: 'Include cluster config',
        buttons: ['Export', 'Cancel']
      })

      if (buttonIndex === 0) {
        const objectToExport = exportCluster(this.$store.state, this.cluster.id, { includeConfig: checkboxChecked })
        const filename = await showSaveDialog({ defaultName: `cluster-${this.cluster.name}.kpf-export.json` })

        if (filename) {
          try {
            saveObjectToJsonFile(objectToExport, filename)
          } catch (error) {
            console.error(error)
            showErrorBox(`Sorry, an error occurred while saving the file. Raw error message: ${error.message}`)
          }
        }
      }
    }
  }
}

</script>

<style lang="scss">
@import "../../assets/styles/variables";

.cluster-item {
  border: $border;
  border-radius: $border-radius;
}

.cluster-item_hidden {
  .cluster-item__header {
    padding-right: 10px;
    border: none
  }
}

.cluster-item__header {
  background: $bg-secondary;
  font-size: $font-size-big;
  font-weight: 500;
  color: $color-text-secondary;
  height: 36px;
  padding: 0 0 0 15px;
  display: flex;
  align-items: center;
  border-bottom: $border;

  .icon_dotes {
    box-sizing: content-box;
    padding: 0 10px;
    cursor: pointer;
    color: $color-text-tertiary;
  }

  .icon_arrow-dropdown {
    cursor: pointer;
  }
}

.cluster-item__empty {
  color: $color-text-secondary;
  padding: 21px;
  text-align: center;
}

.cluster-item__services {
  .service-item + .service-item {
    border-top: $border;
  }
}
</style>
