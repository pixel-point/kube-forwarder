<template>
  <input
    :class="className"
    :value="modelValue"
    v-bind="$attrs"
    @input="onInput"
  >
</template>

<script>
export default {
  name: 'BaseInput',
  inheritAttrs: false,
  props: {
    modelValue: { type: null, default: undefined },
    modelModifiers: { type: Object, default: () => ({}) },
    size: { type: String, default: 'm', validator: val => ['s', 'm'].includes(val) },
    inline: { type: Boolean, default: false },
    invalid: { type: Boolean, default: false }
  },
  emits: ['update:modelValue'],
  computed: {
    className() {
      return {
        'base-input': true,
        [`base-input_size_${this.size}`]: true,
        [`base-input_inline`]: this.inline,
        [`base-input_invalid`]: this.invalid
      }
    }
  },
  methods: {
    // Vue 3 no longer applies v-model .trim/.number on components; the child does.
    onInput(event) {
      let value = event.target.value
      if (this.modelModifiers.number) {
        const parsed = parseFloat(value)
        value = isNaN(parsed) ? value : parsed
      } else if (this.modelModifiers.trim) {
        value = value.trim()
      }
      this.$emit('update:modelValue', value)
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
    color: $color-text-secondary;
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
