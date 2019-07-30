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
    active: { type: Boolean, default: false },
    type: { type: String, default: 'button' },
    loading: { type: Boolean, default: false },
    disabled: { type: Boolean, default: false },
    disabledStyle: { type: Boolean, default: false },
    size: { type: String, default: 'm', validator: val => ['s', 'm', 'l'].includes(val) },
    layout: { type: String, default: 'filled', validator: val => ['filled', 'outline', 'text'].includes(val) },
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
        button_loading: this.loading,
        button_active: this.active,
        button_disabled: this.disabled || this.disabledStyle,
        [`button_layout_${this.layout}`]: true,
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

.buttons {
  font-size: 0;

  .button + .button {
    margin-left: 10px;
  }
}

.button {
  background: none;
  display: inline-block;
  text-transform: uppercase;
  cursor: pointer;
  font-weight: 500;
  border-radius: $border-radius;
  text-decoration: none;
  flex-shrink: 0;
  text-align: center;

  // border exists always to prevent twitches
  border: $border-width solid transparent;

  transition: background-color $hover-transition-speed, color $hover-transition-speed, border-color $hover-transition-speed;

  & > span + svg,
  & > svg + span {
    margin-left: 7px;
  }

  &:focus {
    outline: none;
  }
}

@each $size in $sizes {
  $height: map-get($heights, $size);
  $padding: map-get($paddings, $size);
  $font-size: map-get($font-sizes, $size);

  .button_size_#{$size} {
    height: $height;
    padding: 0 $padding;
    font-size: $font-size;
    line-height: $height - $border-width * 2;
  }
}

.button_theme_default {
  &.button_layout_text,
  &.button_layout_outline {
    color: $color-text-quaternary;

    &:hover, &.button_active {
      color: $color-text-secondary;
    }

    &.button_loading:before {
      border-color: $color-text-quaternary;
    }
  }

  &.button_layout_outline {
    border-color: $border-color;

    &:hover, &.button_active {
      border-color: $color-text-tertiary;
    }
  }
}

.button_layout_filled {
  color: #fff;
}

@each $theme in $button-themes {
  $theme-color: map-get($colors, $theme);

  .button_theme_#{$theme} {
    &.button_layout_filled {
      background-color: $theme-color;

      &:hover, &.button_active {
        background-color: mix($theme-color, #000, 90%);
      }
    }

    &.button_layout_text,
    &.button_layout_outline {
      color: $theme-color;

      &.button_loading:before {
        border-color: $theme-color;
      }
    }

    &.button_layout_outline {
      border-color: rgba($theme-color, 0.5);

      &:hover, &.button_active {
        background-color: rgba($theme-color, $button-outline-hover-bg-opacity);
      }
    }
  }
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
