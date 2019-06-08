<template>
  <form
    :class="{ 'base-form': true, 'base-form_multicolumn': multicolumn }"
    @submit.prevent="$emit('submit', $event)"
  >
    <slot />
  </form>
</template>

<script>
export default {
  props: {
    multicolumn: { type: Boolean, default: false }
  }
}
</script>

<style lang="scss">
@import "../../../assets/styles/variables";

.base-form {
  background-color: rgba($color-text, 0.04);
  padding: 20px;
  border-radius: $border-radius;
  font-size: 0; // to remove empty spaces

  .control-hint {
    color: rgba($color-text, 0.5);
    font-size: $font-size-small;
    font-style: italic;
    float: right
  }

  .controls {
    > .base-input, .base-textarea, .base-select {
      width: 100%;
    }

    > .base-textarea {
      max-width: 100%;
      min-width: 100%;
      min-height: 100px;
    }
  }

  .control-actions {
    display: flex;
    align-items: center;
    margin-top: 30px;

    .error-message {
      color: #ff4164;
      margin-left: 15px;
    }
  }

  .control-actions__error {
    font-size: $font-size-small;
    color: $color-danger;
    padding: 0 10px
  }

  .control-actions__message {
    font-size: $font-size-small;
    padding: 0 10px
  }
}

.base-form_multicolumn {
  font-size: 0;
  padding: 20px 5px;

  > * {
    font-size: $font-size-base;
  }

  .control-group,
  .control-actions {
    padding: 0 15px;
  }

  .control-group_size_2 {
    display: inline-block;
    width: 50%;
  }
}

</style>
