<template>
  <textarea
    class="base-textarea"
    :value="value"
    @input="$emit('input', $event.target.value)"
  />
</template>

<script>
export default {
  name: 'BaseTextArea',
  props: {
    value: { type: String, default: '' }
  }
}
</script>

<style lang="scss">
@import "../../../assets/styles/mixins";

textarea {
  --border-color: #{rgba(map-get($color-text, "light"), 0.15)};
  @include theme("dark") {
    --border-color: #{rgba(map-get($color-text, "dark"), 0.15)};
  }

  color: var(--color-text);
  border: 1px solid var(--border-color);
  border-radius: $border-radius;
  padding: 8px 15px;
  height: 40px;
  line-height: 1.71;
  font-size: $font-size-base;
  background-color: var(--body-background-color);
}

textarea:focus {
  border-color: $color-primary;
  outline: none;
}

textarea::placeholder {
  color: rgba($color-text-theme-light, 0.6);
}

.control-group_error .base-textarea {
  border: 1px solid $color-danger;
}
</style>
