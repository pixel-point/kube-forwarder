<template>
  <div class="autocomplete-input">
    <BaseInput v-bind="$attrs" :value="value" v-on="listeners" />
    <Popup v-if="(matchedOptions.length || loading) && focused" :position="'bottom'" :align="'both'">
      <div v-if="loading" class="autocomplete-input__loader-container">
        <Loader size="s" />
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
    value: BaseInput.props.value // eslint-disable-line vue/require-default-prop
  },
  data() {
    return {
      focused: false
    }
  },
  computed: {
    matchedOptions() {
      return this.options.filter(x => x.length !== this.value.length && x.startsWith(this.value))
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
.autocomplete-input {
  position: relative;

  .base-input {
    width: 100%;
  }

  .popup__actions {
    max-height: 200px;
    overflow: auto;
  }
}

.autocomplete-input__loader-container {
  height: 40px;
}
</style>
