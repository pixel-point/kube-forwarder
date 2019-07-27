<template>
  <div :class="className">
    <Button
      v-for="option in options"
      :key="option[0]"
      v-bind="buttonProps(option)"
      @click="setValue(option[0])"
    >
      {{ option[1] }}
    </Button>
  </div>
</template>

<script>
import Button from './Button'

export default {
  name: 'RadioButtonGroup',
  components: { Button },
  model: { event: 'change' },
  props: {
    theme: Button.props.theme, // eslint-disable-line vue/require-default-prop
    options: { type: Array, required: true },
    value: { type: null, default: null }
  },
  computed: {
    className() {
      return {
        'radio-button-group': true
      }
    }
  },
  methods: {
    buttonProps(option) {
      const { theme } = this
      const customProps = option[2] || {}
      return { theme, ...customProps, class: 'radio-button-group__button', active: this.value === option[0] }
    },
    setValue(value) {
      this.$emit('change', value)
    }
  }
}
</script>

<style lang="scss">
@import "../../assets/styles/variables";

.radio-button-group__button {
  border-radius: 0;

  + .radio-button-group__button {
    border-left-color: rgba($color-text, 0.5);
  }

  &:first-child {
    border-radius: $border-radius 0 0 $border-radius;
  }

  &:last-child {
    border-radius: 0 $border-radius $border-radius 0;
  }
}

</style>
