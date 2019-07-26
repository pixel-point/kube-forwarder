<template>
  <div class="service-item">
    <div :class="`service-item__status service-item__status_state_${serviceState}`">
      <Hint showOn="hover" :offset="-7.5">{{ serviceState === 'connected' ? 'Running' : 'Stopped' }}</Hint>
    </div>

    <div class="service-item__content">
      <div class="service-item__title">{{ getServiceLabel(service) }}</div>
      <div class="service-item__description">
        <span>From <span class="service-item__namespace">{{ service.namespace }}</span> namespace exposed to</span>
        <span class="service-item__ports">
          <span v-for="(forward, index) in forwards" :key="forward.id" class="service-item__port">
            <b>{{ forward.localPort }}</b><span v-if="index !== forwards.length - 1"
                                                class="service-item__port-divider" />
          </span>
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

      <Dropdown :popup-props="{ offsetHorizontal: 1 }">
        <template v-slot:trigger="triggerSlotProps">
          <Button class="service-item__action-more" layout="text" @click="triggerSlotProps.toggle">
            <IconDotes />
          </Button>
        </template>

        <ul class="popup__actions">
          <li><Action :disabled-style="!canModify" @click="handleEdit">Edit</Action></li>
          <li><Action :to="servicePath('/clone')">Clone</Action></li>
          <li><Action theme="danger" :disabled-style="!canModify" @click="handleDelete">Delete</Action></li>
        </ul>
      </Dropdown>
    </div>
  </div>
</template>

<script>
import { CONNECTED } from '../../lib/constants/connection-states'
import { showMessageBox, showConfirmBox } from '../../lib/helpers/ui'
import { getServiceLabel } from '../../lib/helpers/service'

import Dropdown from '../shared/Dropdown'
import IconDotes from '../shared/icons/IconDotes'
import Button from '../shared/Button'
import IconPause from '../shared/icons/IconPause'
import IconPlay from '../shared/icons/IconPlay'
import Action from '../shared/Action'
import Hint from '../shared/Hint'

export default {
  components: {
    Button,
    Dropdown,
    IconDotes,
    IconPause,
    IconPlay,
    Action,
    Hint
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
    getServiceLabel,
    handleStartStop() {
      if (this.serviceState === 'ready') this.startService()
      else if (this.serviceState === 'connected') this.stopService()
      else this.showBlockedReason()
    },
    async startService() {
      const { success, error, results } = await this.$store.dispatch('Connections/createConnection', this.service)

      if (!success) {
        if (error) {
          return showMessageBox(error.message, { detail: error.originError && error.originError.message })
        }

        if (results) {
          const messages = results.filter(x => !x.success)
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
        const serviceName = service ? getServiceLabel(service) : `Service(serviceId:${state.serviceId})`
        return `${state.port} used by ${serviceName}`
      }).join(';\n')

      showMessageBox(`Service can't be forwarded. Some ports already in use.\n\n${portsDetails}`)
    },
    async handleDelete() {
      if (!this.canModify) {
        return showMessageBox('You must stop forwarding before deleting the service')
      }

      const confirm = await showConfirmBox(`Are you sure to delete "${getServiceLabel(this.service)}" service`)
      if (confirm) {
        this.$store.dispatch('Services/deleteService', this.service.id)
      }
    },
    handleEdit() {
      if (!this.canModify) {
        return showMessageBox('You must stop forwarding before editing the service')
      }

      this.$router.push(this.servicePath('/edit'))
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
  position: relative;
  cursor: default;
}

.service-item__status_state_connected {
  background-color: $color-secondary;
}

.service-item__content {
  margin: 0 17px
}

.service-item__title {
  font-size: $font-size-big;
  margin-bottom: 5px;
  font-weight: 500;
}

.service-item__namespace {
  color: $color-text
}

.service-item__description {
  color: $color-text-tertiary;

  b {
    color: $color-text
  }
}

.service-item__port {
  &:first-child {
    padding-left: 2.5px;
  }

  &:last-child {
    padding-right: 2.5px;
  }
}

.service-item__port-divider {
  width: 3px;
  height: 3px;
  display: inline-block;
  background-color: $color-text-quaternary;
  margin: -1px 5px 0;
  border-radius: 50%;
  vertical-align: middle;
}

.service-item__actions {
  display: flex;
  align-items: center;
}

.service-item__action-more {
  padding: 0;
  width: 24px;
  margin-left: 14px;
}

.service-item__action-button {
  padding: 0;
  width: 30px;
}
</style>
