<template>
  <component
    :is="tag"
    :class="className"
    v-bind="computedProps"
    v-on="$listeners"
  >
    <slot />
  </component>
</template>

<script>
export default {
  name: 'Button',
  props: {
    to: { type: String, default: null },
    href: { type: String, default: null },
    type: { type: String, default: 'button' },
    loading: { type: Boolean, default: false },
    borderless: { type: Boolean, default: false },
    outline: { type: Boolean, default: false },
    disabled: { type: Boolean, default: false },
    disabledStyle: { type: Boolean, default: false },
    size: { type: String, default: 'm', validator: val => ['s', 'm', 'l'].includes(val) },
    theme: {
      type: String,
      default: 'primary',
      validator: val => ['primary', 'danger'].includes(val)
    }
  },
  computed: {
    bordered() {
      return !this.borderless && this.outline
    },
    tag() {
      if (this.to) return 'router-link'
      if (this.href) return 'a'
      return 'button'
    },
    className() {
      return {
        button: true,
        button_bordered: this.bordered,
        button_outline: this.outline,
        button_loading: this.loading,
        button_disabled: this.disabled || this.disabledStyle,
        [`button_theme_${this.theme}`]: true,
        [`button_size_${this.size}`]: true
      }
    },
    computedProps() {
      const { href, disabled, to, tag, type } = this

      if (tag === 'router-link') return { to }
      if (tag === 'a') return { href, disabled }
      return { disabled, type }
    }
  }
}
</script>

<style lang="scss">
@import "../../assets/styles/mixins";

.button {
  background: none;
  border: none;
  display: inline-block;
  text-transform: uppercase;
  cursor: pointer;
  font-weight: 500;
  border-radius: $border-radius;
  color: #fff;
  text-decoration: none;
  flex-shrink: 0;

  & > span + svg,
  & > svg + span {
    margin-left: 7px;
  }

  &:focus {
    outline: none;
  }
}

.button_size_s {
  height: 30px;
  line-height: 30px;
  padding: 0 10px;
  font-size: $font-size-small;

  &.button_bordered {
    line-height: 28px;
  }
}

.button_size_m {
  height: 36px;
  line-height: 36px;
  padding: 0 15px;
  font-size: $font-size-small;

  &.button_bordered {
    line-height: 34px;
  }
}

.button_size_l {
  height: 50px;
  line-height: 50px;
  padding: 0 25px;
  font-size: 17px;

  &.button_bordered {
    line-height: 48px;
  }
}

.button_bordered {
  border: 1px solid transparent;
}

.button_theme_primary {
  background-color: $color-primary;
  border-color: rgba($color-primary, 0.5);

  @include hf {
    background-color: #2364cd;
  }

  &.button_outline {
    color: $color-primary;

    @include hf {
      background-color: rgba($color-primary, $button-outline-hover-bg-opacity);
    }

    &.button_loading:before {
      border-color: $color-primary;
    }
  }
}

.button_theme_danger {
  background-color: $color-danger;
  border-color: $color-danger;

  @include hf {
    background-color: #dd3e60;
  }

  &.button_outline {
    color: $color-danger;

    @include hf {
      background-color: rgba(255, 65, 100, $button-outline-hover-bg-opacity);
    }
  }
}

.button_outline {
  background-color: transparent;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.button_loading {
  color: transparent !important;
  text-align: center;
  position: relative;
  pointer-events: none;

  &:before {
    content: "";
    display: inline-block;
    position: absolute;
    top: 50%;
    margin: -9px 0 0 -9px;
    left: 50%;
    width: 16px;
    height: 16px;
    border: 1px solid #fff;
    border-radius: 50%;
    border-bottom-color: transparent !important;
    animation: spin 1s linear infinite;
  }
}

.button[disabled] {
  cursor: not-allowed;
}

.button_disabled {
  opacity: 0.25;
}
</style>
