<template>
  <div class="autocomplete-input">
    <BaseInput v-bind="$attrs" :value="value" v-on="listeners" />
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
  props: {
    loading: { type: Boolean, default: false },
    options: { type: Array, default: () => [] },
    value: BaseInput.props.value, // eslint-disable-line vue/require-default-prop
    notFoundMessage: { type: String, default: 'Not found' }
  },
  data() {
    return {
      focused: false
    }
  },
  computed: {
    matchedOptions() {
      return this.options.filter(x => x.startsWith(this.value))
    },
    listeners() {
      return {
        ...this.$listeners,
        focus: this.handleFocus,
        blur: this.handleBlur
      }
    }
  },
  methods: {
    handleOptionClick(option) {
      this.$emit('input', option)
    },
    handleFocus() {
      this.focused = true
      this.$listeners.focus && this.$listeners.focus()
    },
    handleBlur() {
      setTimeout(() => {
        this.focused = false
        this.$listeners.blur && this.$listeners.blur()
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
