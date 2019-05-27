<template>
  <div v-click-outside="close" class="dropdown">
    <slot name="trigger" :toggle="toggle" :opened="opened" />

    <Popup
      v-if="opened"
      :position="position"
      :align="align"
      v-bind="popupProps"
    >
      <slot :close="close"/>
    </Popup>
  </div>
</template>

<script>
import ClickOutside from 'vue-click-outside'

import Popup from './Popup'

export default {
  components: {
    Popup
  },
  directives: {
    ClickOutside
  },
  props: {
    popupProps: { type: Object, default: () => ({}) }
  },
  data() {
    return {
      opened: false
    }
  },
  computed: {
    position() {
      return this.popupProps.position || 'bottom'
    },
    align() {
      return this.popupProps.align || 'right'
    }
  },
  methods: {
    toggle() {
      this.opened = !this.opened
    },
    close() {
      this.opened = false
    }
  }
}
</script>

<style lang="scss">
.dropdown {
  position: relative;
  display: inline-block;
}
</style>
