<template>
  <component :is="tag" :class="className" v-bind="attrs" v-on="listeners">
    <slot />
  </component>
</template>

<script>
export default {
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
    attrs() {
      const { to, disabled } = this
      return disabled ? {} : { to }
    },
    listeners() {
      return this.disabled ? {} : this.$listeners
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
  color: $color-text-secondary;
}
</style>
