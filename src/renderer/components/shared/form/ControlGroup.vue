<template>
  <div :class="className">
    <label v-if="label" class="control-label">{{ label }}</label>
    <div v-if="hint" class="control-hint">{{ hint }}</div>
    <div class="controls">
      <slot v-bind="slotProps" />
    </div>
    <ValidationErrors v-if="attribute" :attribute="attribute" />
  </div>
</template>

<script>
import ValidationErrors from './ValidationErrors'

export default {
  components: { ValidationErrors },
  props: {
    label: { type: String, default: null },
    hint: { type: String, default: null },
    attribute: { type: Object, default: null },
    size: { type: String, default: '1', validator: val => ['1', '2'].includes(val) },
    disabled: { type: Boolean, default: false }
  },
  computed: {
    className() {
      return {
        'control-group': true,
        'control-group_error': this.attribute && this.attribute.$error,
        [`control-group_size_${this.size}`]: true,
        [`control-group_disabled`]: this.disabled
      }
    },
    slotProps() {
      const { attribute, disabled } = this
      return { attribute, disabled }
    }
  }
}
</script>

<style lang="scss">
@import "../../../assets/styles/variables";

.control-group {
  & + .control-group {
    margin-top: 30px;
  }

  > .validation-errors {
    display: none;
    position: absolute;
    padding-top: 2px;
  }
}

.control-group_error {
  > .validation-errors {
    display: block;
  }
}

.control-group_disabled {
  .control-label {
    color: $color-text-tertiary
  }
}
</style>
