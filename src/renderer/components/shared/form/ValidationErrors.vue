<template>
  <div v-if="attribute.$error" class="validation-errors">
    <div v-if="attribute.required === false" class="validation-errors__error">Field is required.</div>
    <div v-if="attribute.integer === false" class="validation-errors__error">Field must be an integer.</div>
    <div v-if="attribute.between === false" class="validation-errors__error">
      Field must be between specific values.
    </div>
  </div>
</template>

<script>
export default {
  props: {
    attribute: { type: Object, required: true }
  }
}
</script>

<style lang="scss">
@import "../../../assets/styles/variables";

.validation-errors__error {
  font-size: $font-size-small;
  color: $color-danger;
}
</style>
