<template>
  <div class="service-item">
    <div :class="`service-item__status service-item__status_state_${serviceState}`" />

    <div class="service-item__description">
      <div class="service-item__title">
        <span v-if="service.alias">{{ service.alias }}</span>
        <span v-else>
          {{ service.workloadName }}
          <span class="text text_theme_secondary"> from </span>
          {{ service.namespace }}
        </span>
      </div>
      <div class="service-item__ports">
        <span>Exposed to</span>
        <span v-for="forward in forwards" :key="forward.id" class="service-item__port">
          <b>{{ forward.localPort }}</b>
        </span>
        <span>port{{ forwards.length > 1 ? 's' : '' }}</span>
      </div>
    </div>

    <div class="space" />

    <div class="service-item__actions">
      <Button
        theme="primary"
        size="s"
        class="service-item__action-button"
        :layout="serviceState === 'connected' ? 'outline' : 'filled'"
        :disabled-style="serviceState === 'blocked'"
        :loading="serviceState === 'connecting'"
        @click="handleStartStop"
      >
        <IconPause v-if="serviceState === 'connected'" />
        <IconPlay v-else />
      </Button>

      <Dropdown>
        <template v-slot:trigger="triggerSlotProps">
          <IconDotes @click.native="triggerSlotProps.toggle" />
        </template>

        <ul class="popup__actions">
          <li><Action :to="servicePath('/edit')" :disabled="!canModify">Edit</Action></li>
          <li><Action :to="servicePath('/clone')">Clone</Action></li>
          <li><Action theme="danger" :disabled="!canModify" @click="handleDelete">Delete</Action></li>
        </ul>
      </Dropdown>
    </div>
  </div>
</template>

<script>
import { CONNECTED } from '../../lib/constants/connection-states'
import { showMessageBox, showConfirmBox } from '../../lib/helpers/ui'

import Dropdown from '../shared/Dropdown'
import IconDotes from '../shared/icons/IconDotes'
import Button from '../shared/Button'
import IconPause from '../shared/icons/IconPause'
import IconPlay from '../shared/icons/IconPlay'
import Action from '../shared/Action'

export default {
  components: {
    Button,
    Dropdown,
    IconDotes,
    IconPause,
    IconPlay,
    Action
  },
  props: {
    service: { type: Object, required: true }
  },
  computed: {
    forwards() {
      return this.service.forwards
    },
    portStates() {
      return this.forwards.map(forward => this.$store.state.Connections[forward.localPort] || {})
    },
    serviceState() {
      const isFree = this.portStates.every(state => !state.serviceId)
      if (isFree) return 'ready'

      const isActive = this.portStates.every(state => state.serviceId === this.service.id)
      if (isActive) {
        const isConnected = this.portStates.every(state => state.state === CONNECTED)
        return isConnected ? 'connected' : 'connecting'
      }

      return 'blocked'
    },
    canModify() {
      return this.serviceState === 'blocked' || this.serviceState === 'ready'
    },
    portStatesAlreadyInUse() {
      return this.portStates.filter(state => state.serviceId && state.serviceId !== this.service.id)
    }
  },
  methods: {
    handleStartStop() {
      if (this.serviceState === 'ready') this.startService()
      else if (this.serviceState === 'connected') this.stopService()
      else this.showBlockedReason()
    },
    async startService() {
      const result = await this.$store.dispatch('Connections/createConnection', this.service)

      if (!result.success) {
        if (result.message) return showMessageBox(result.message)
        if (result.results) {
          const messages = result.results.filter(x => !x.success)
            .map(x => `Failed to forward port ${x.forward.localPort} to ${x.forward.remotePort}  - ${x.error}`)

          showMessageBox(messages.join(';\n'))
        }
      }
    },
    stopService() {
      this.$store.dispatch('Connections/deleteConnection', this.service)
    },
    showBlockedReason() {
      const portsDetails = this.portStatesAlreadyInUse.map(state => {
        const service = this.$store.state.Services.items[state.serviceId]
        const serviceName = service ? service.name : `Service(serviceId:${state.serviceId})`
        return `${state.port} used by ${serviceName}`
      }).join(';\n')

      showMessageBox(`Service can't be forwarded. Some ports already in use.\n\n${portsDetails}`)
    },
    async handleDelete() {
      const confirm = await showConfirmBox(`Are you sure to delete "${this.service.name}" service`)
      if (confirm) {
        this.$store.dispatch('Services/deleteService', this.service.id)
      }
    },
    servicePath(postfix = '') {
      return `/clusters/${this.service.clusterId}/services/${this.service.id}${postfix}`
    }
  }
}
</script>

<style lang="scss">
@import "../../assets/styles/mixins";

.service-item {
  padding: 10px 0 10px 15px;
  display: flex;

  b {
    font-weight: 500;
  }
}

.service-item__status {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background-color: $color-danger;
  margin-top: 5px;
}

.service-item__status_state_connected {
  background-color: $color-secondary;
}

.service-item__description {
  margin: 0 17px
}

.service-item__title {
  font-size: $font-size-big;
  margin-bottom: 5px;
}

.service-item__ports {
  color: $color-text-secondary;

  b {
    color: $color-text
  }
}

.service-item__port + .service-item__port:before {
  content: ', '
}

.service-item__actions {
  display: flex;
  align-items: center;

  .icon_dotes {
    box-sizing: content-box;
    padding: 0 10px;
    cursor: pointer;
    color: $color-text-tertiary;
    margin-left: 10px;
  }
}

.service-item__action-button {
  padding: 0;
  width: 30px;
}
</style>
