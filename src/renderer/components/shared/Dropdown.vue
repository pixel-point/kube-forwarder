<template>
  <div v-click-outside="close" :class="className">
    <slot name="trigger" :toggle="toggle" :opened="opened" />

    <Popup
      v-if="opened"
      ref="popup"
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
    className() {
      return {
        dropdown: true,
        dropdown_opened: this.opened
      }
    },
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
      if (this.opened) {
        this.$nextTick(() => this.$refs.popup.$el.scrollIntoView())
      }
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
