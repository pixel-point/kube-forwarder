<template>
  <input
    :class="className"
    :value="value"
    v-on="{
      ...$listeners,
      input: event => $emit('input', event.target.value)
    }"
  >
</template>

<script>
export default {
  name: 'BaseInput',
  props: {
    value: { type: null, default: undefined },
    size: { type: String, default: 'm', validator: val => ['s', 'm'].includes(val) },
    inline: { type: Boolean, default: false },
    invalid: { type: Boolean, default: false },
  },
  computed: {
    className() {
      return {
        'base-input': true,
        [`base-input_size_${this.size}`]: true,
        [`base-input_inline`]: this.inline,
        [`base-input_invalid`]: this.invalid
      }
    }
  }
}
</script>

<style lang="scss">
@import "../../../assets/styles/mixins.scss";

.base-input {
  --border-color: #{rgba(map-get($color-text, "light"), 0.15)};
  @include theme("dark") {
    --border-color: #{rgba(map-get($color-text, "dark"), 0.15)};
  }

  color: var(--color-text);
  border: 1px solid var(--border-color);
  border-radius: $border-radius;
  padding: 9px 15px 10px;
  line-height: 16px;
  background-color: var(--body-background-color);
  font-weight: 500;
  font-size: $font-size-base;
  transition: border-color $hover-transition-speed;

  &:focus {
    border-color: $color-primary;
    outline: none;
  }

  &[type="search"] {
    -webkit-appearance: none;
  }

  &::placeholder {
    color: var(--color-text-secondary);
    font-weight: normal;
  }

  // Remove caret at screenshots.
  .body_env_test & {
    color: transparent;
    text-shadow: 0 0 0 red;
  }
}

.base-input_size_m {
  height: 40px;
}

.base-input_size_s {
  height: 36px;
}

.base-input_inline {
  border: none;
  background: none;
}

.base-input_invalid,
.control-group_error .base-input {
  border: 1px solid $color-danger;

  &.base-input_inline {
    border: none;
    background-color: $bg-danger;
  }
}

.base-input[disabled] {
  opacity: 0.5;
}

input[type=number]::-webkit-inner-spin-button,
input[type=number]::-webkit-outer-spin-button {
  -webkit-appearance: none;
  margin: 0;
}
</style>
