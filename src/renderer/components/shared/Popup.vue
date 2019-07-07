<template>
  <div :class="className" :style="style">
    <slot />
  </div>
</template>

<script>
export default {
  name: 'Popup',
  props: {
    position: { type: String, required: true, validator: x => ['bottom'].includes(x) },
    align: { type: String, required: true, validator: x => ['center', 'right', 'both'].includes(x) },
    width: { type: null, default: null },
    offsetVertical: { type: Number, default: 5 },
    offsetHorizontal: { type: Number, default: 0 }
  },
  computed: {
    className() {
      return {
        popup: true,
        [`popup_position_${this.position}`]: true,
        [`popup_align_${this.align}`]: true
      }
    },
    style() {
      const result = {}

      result.transform = `translate3d(${this.offsetHorizontal}px, ${this.offsetVertical}px, 0px)`

      if (this.position === 'bottom') {
        if (this.align === 'right') {
          result.width = `${this.width}px`
        }

        if (this.align === 'center') {
          result.marginLeft = `-${this.width / 2}px`
        }
      }

      return result
    }
  }
}
</script>

<style scoped lang="scss">
@import "../../assets/styles/variables";

.popup {
  position: absolute;
  background: var(--body-background-color);
  z-index: 1;
  border-radius: $border-radius;
  box-shadow: 0 2px 4px 0 var(--border-color);
  border: solid 1px var(--border-color);
}

.popup_position_bottom {
  top: 100%;
}

.popup_position_bottom.popup_align_center {
  left: 50%;
}

.popup_position_bottom.popup_align_right {
  right: 0;
}

.popup_position_bottom.popup_align_both {
  right: 0;
  left: 0;
}

.popup__actions {
  margin: 0;
  padding: 0;
  list-style: none;

  li + li {
    border-top: $border;
  }

  .action {
    padding: 0 30px 0 15px;
    height: 36px;
    line-height: 36px;
    display: block;
    white-space: nowrap;

    transition: background-color $hover-transition-speed;

    &:hover {
      background-color: var(--bg-secondary);
    }
  }
}
</style>
