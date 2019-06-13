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
@import "../../../assets/styles/_variables.scss";

.base-input {
  color: $color-text;
  border: 1px solid rgba($color-text, 0.15);
  border-radius: $border-radius;
  padding: 9px 15px 10px;
  line-height: 16px;
  background: #fff;
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
    color: $color-text-placeholder;
    font-weight: normal;
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
