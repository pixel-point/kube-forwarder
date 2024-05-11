<template>
  <div v-show="show" class="hint" :class="{ [`align-${align}`]: true, [`position-${position}`]: true }" :style="style">
    <div class="triangle" :style="triangleStyle" />
    <slot />
  </div>
</template>

<script>
import * as validators from '../../lib/prop-validators'

const TRIANGLE_WIDTH = 10

export default {
  name: 'Hint',
  props: {
    position: { type: String, validator: validators.oneOf(['top', 'bottom']), default: 'top' },
    align: { type: String, validator: validators.oneOf(['right', 'left', 'center']), default: 'left' },
    showOn: { type: String, default: '', validator: validators.oneOf(['hover']) },
    offset: { type: Number, default: 0 }
  },
  data() {
    return {
      show: true,
      addedListeners: false,
      el: null,
      parent: null
    }
  },
  computed: {
    style() {
      if (['right', 'left'].includes(this.align)) {
        return { [`${this.align}`]: `${this.offset}px` }
      }
      return ''
    },
    triangleStyle() {
      // this.show required for this.el.parentNode.offsetWidth "binding"
      if (this.show && this.el && ['right', 'left'].includes(this.align)) {
        return { [`${this.align}`]: `${this.el.parentNode.offsetWidth / 2 - TRIANGLE_WIDTH / 2 - this.offset}px` }
      }
      return ''
    }
  },
  mounted() {
    this.el = this.$el
    this.parent = this.$el.parentNode

    if (this.showOn === 'hover') {
      this.addedListeners = true
      this.show = false
      this.parent.addEventListener('mouseenter', this.onMouseEnter)
      this.parent.addEventListener('mouseleave', this.onMouseLeave)
    }
  },
  destroyed() {
    if (this.addedListeners) {
      this.parent.removeEventListener('mouseenter', this.onMouseEnter)
      this.parent.removeEventListener('mouseleave', this.onMouseLeave)
    }
  },
  methods: {
    onMouseEnter() {
      this.show = true
    },
    onMouseLeave() {
      this.show = false
    }
  }
}
</script>

<style scoped lang="scss">
@import "../../assets/styles/variables";

.hint {
  position: absolute;
  max-width: 300px;
  background: rgba($color-text, 0.9);
  border-radius: 2px;
  padding: 6px 8px;
  color: #fff;
  font-size: 13px;
  z-index: 1000;

  &.position-top {
    margin-bottom: 10px;
    bottom: 100%;

    .triangle {
      bottom: -5px;
      border-bottom-width: 0;
      border-top-color: rgba($color-text, 0.9);
    }
  }

  &.position-bottom {
    margin-top: 10px;
    top: 100%;

    .triangle {
      top: -5px;
      border-top-width: 0;
      border-bottom-color: rgba($color-text, 0.9);
    }
  }

  &.align-center {
    left: 50%;
    transform: translateX(-50%);

    .triangle {
      left: 50%;
      margin-left: -5px;
    }
  }
}

.triangle {
  position: absolute;
  width: 0;
  height: 0;
  border-style: solid;
  border-width: 5px;
  border-color: transparent;
}
</style>
