<template>
  <textarea
    class="base-textarea"
    :value="modelValue"
    @input="onInput"
  />
</template>

<script>
export default {
  name: 'BaseTextArea',
  props: {
    modelValue: { type: String, default: '' },
    modelModifiers: { type: Object, default: () => ({}) }
  },
  emits: ['update:modelValue'],
  methods: {
    onInput(event) {
      let value = event.target.value
      if (this.modelModifiers.trim) value = value.trim()
      this.$emit('update:modelValue', value)
    }
  }
}
</script>

<style lang="scss">
@import "../../../assets/styles/_variables.scss";

textarea {
  color: $color-text;
  border: 1px solid rgba($color-text, 0.15);
  border-radius: $border-radius;
  padding: 8px 15px;
  height: 40px;
  line-height: 1.71;
  font-size: $font-size-base;
}

textarea:focus {
  border-color: $color-primary;
  outline: none;
}

textarea::placeholder {
  color: rgba($color-text, 0.6);
}

.control-group_error .base-textarea {
  border: 1px solid $color-danger;
}
</style>
