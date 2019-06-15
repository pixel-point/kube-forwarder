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
      <slot :close="close" />
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
        this.$nextTick(() => {
          const popupEl = this.$refs.popup.$el

          if (!this.isInViewport(popupEl)) {
            popupEl.scrollIntoView({ behavior: 'smooth', block: 'center' })
          }
        })
      }
    },
    close() {
      this.opened = false
    },
    isInViewport(elem) {
      const bounding = elem.getBoundingClientRect()

      return (
        bounding.top >= 0 &&
        bounding.left >= 0 &&
        bounding.bottom <= (window.innerHeight || document.documentElement.clientHeight) &&
        bounding.right <= (window.innerWidth || document.documentElement.clientWidth)
      )
    }
  }
}
</script>

<style lang="scss">
.dropdown {
  position: relative;
  display: inline-block;

  .popup {
    margin-bottom: 20px;
  }
}
</style>
