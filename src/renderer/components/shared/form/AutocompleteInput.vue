<template>
  <div class="autocomplete-input">
    <BaseInput
      v-bind="$attrs"
      :model-value="modelValue"
      @update:model-value="value => $emit('update:modelValue', value)"
      @focus="handleFocus"
      @blur="handleBlur"
    />
    <Popup v-if="focused && options.length > 0" :position="'bottom'" :align="'both'">
      <div v-if="loading" class="autocomplete-input__content">
        <Loader size="s" />
      </div>

      <div v-else-if="matchedOptions.length === 0" class="autocomplete-input__content">
        {{ notFoundMessage }}
      </div>

      <ul v-else class="popup__actions">
        <li v-for="option in matchedOptions" :key="option">
          <Action @mousedown="handleOptionClick(option)">{{ option }}</Action>
        </li>
      </ul>
    </Popup>
  </div>
</template>

<script>
import BaseInput from './BaseInput'
import Popup from '../Popup'
import Action from '../Action'
import Loader from '../Loader'

export default {
  name: 'AutocompleteInput',
  components: { BaseInput, Popup, Action, Loader },
  inheritAttrs: false,
  props: {
    loading: { type: Boolean, default: false },
    options: { type: Array, default: () => [] },
    modelValue: { type: null, default: undefined },
    notFoundMessage: { type: String, default: 'Not found' }
  },
  emits: ['update:modelValue', 'focus', 'blur'],
  data() {
    return {
      focused: false
    }
  },
  computed: {
    matchedOptions() {
      return this.options.filter(x => x.startsWith(this.modelValue || ''))
    }
  },
  methods: {
    handleOptionClick(option) {
      this.$emit('update:modelValue', option)
    },
    handleFocus() {
      this.focused = true
      this.$emit('focus')
    },
    handleBlur() {
      setTimeout(() => {
        this.focused = false
        this.$emit('blur')
      }, 100)
    }
  }
}
</script>

<style lang="scss">
@import "../../../assets/styles/variables";

.autocomplete-input {
  font-size: $font-size-base;
  position: relative;

  .base-input {
    width: 100%;
  }

  .popup__actions {
    max-height: 200px;
    overflow: auto;
  }
}

.autocomplete-input__content {
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: $color-text-secondary;

  .loader {
    position: static;
  }
}
</style>
