<template>
  <component :is="tag" :class="className" v-bind="bindings">
    <slot />
  </component>
</template>

<script>
export default {
  inheritAttrs: false,
  props: {
    to: { type: String, default: null },
    disabled: { type: Boolean, default: null },
    disabledStyle: { type: Boolean, default: null },
    theme: { type: String, default: null }
  },
  computed: {
    tag() {
      const { disabled, to } = this

      if (disabled) return 'span'
      if (to) return 'router-link'
      return 'span'
    },
    className() {
      return {
        action: true,
        'action_disabled': this.disabled || this.disabledStyle,
        [`action_theme_${this.theme}`]: this.theme
      }
    },
    // In Vue 3, event listeners live in $attrs. When disabled we drop both the
    // `to` link and any listeners.
    bindings() {
      if (this.disabled) return {}
      return { to: this.to, ...this.$attrs }
    }
  }
}
</script>

<style lang="scss">
@import "../../assets/styles/variables";

.action {
  cursor: pointer;
  color: $color-text;
  font-weight: 500;
  text-decoration: none;
}

.action_disabled {
  cursor: not-allowed;
}

.action_theme_danger {
  color: $color-danger
}

.action_disabled {
  color: $color-text-tertiary;
}
</style>
