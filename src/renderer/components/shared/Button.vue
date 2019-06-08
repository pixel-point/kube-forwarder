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
      default: 'default',
      validator: val => ['default', 'primary', 'danger', 'secondary'].includes(val)
    }
  },
  computed: {
    tag() {
      if (this.to) return 'router-link'
      if (this.href) return 'a'
      return 'button'
    },
    className() {
      return {
        button: true,
        button_bordered: !this.borderless,
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
  text-align: center;

  & > span + svg,
  & > svg + span {
    margin-left: 7px;
  }

  &:focus {
    outline: none;
  }
}

$sizes: s m l;
$heights: (
  "s": 30px,
  "m": 36px,
  "l": 50px
);
$paddings: (
  "s": 10px,
  "m": 15px,
  "l": 25px
);
$font-sizes: (
  "s": $font-size-small,
  "m": $font-size-small,
  "l": $font-size-large,
);

$border-width: 1px;

@each $size in $sizes {
  $height: map-get($heights, $size);
  $padding: map-get($paddings, $size);
  $font-size: map-get($font-sizes, $size);

  .button_size_#{$size} {
    height: $height;
    line-height: $height;
    padding: 0 $padding;
    font-size: $font-size;

    &.button_bordered {
      line-height: $height - $border-width * 2;
    }
  }
}

.button_bordered {
  border: $border-width solid transparent;
}

.button_theme_default {
  border-color: $border-color;

  &.button_outline {
    color: $color-text-tertiary;

    @include hf {
      background-color: rgba($color-text-tertiary, $button-outline-hover-bg-opacity);
    }

    &.button_loading:before {
      border-color: $color-text-tertiary;
    }
  }
}

@each $theme in $button-themes {
  $theme-color: map-get($colors, $theme);

  .button_theme_#{$theme} {
    background-color: $theme-color;
    border-color: rgba($theme-color, 0.5);

    @include hf {
      background-color: mix($theme-color, #000, 90%);
    }

    &.button_outline {
      color: $theme-color;

      @include hf {
        background-color: rgba($theme-color, $button-outline-hover-bg-opacity);
      }

      &.button_loading:before {
        border-color: $theme-color;
      }
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
