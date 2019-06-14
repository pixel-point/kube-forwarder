<template>
  <Dropdown class="base-select" :popup-props="{ align: 'both' }">
    <template v-slot:trigger="triggerSlotProps">
      <div class="base-select__input-wrap" @click="triggerSlotProps.toggle">
        <BaseInput readonly :value="valueLabel" :placeholder="placeholder"/>
        <IconArrowDropdown :to="triggerSlotProps.opened ? 'top' : 'bottom'" />
      </div>
    </template>

    <template v-slot="slotProps">
      <ul class="popup__actions">
        <li v-for="option in options" :key="option[0]">
          <Action @click="selectOption(option, slotProps.close)">{{ option[1] }}</Action>
        </li>
      </ul>
    </template>
  </Dropdown>
</template>

<script>
import Dropdown from '../Dropdown'
import BaseInput from './BaseInput'
import IconArrowDropdown from '../icons/IconArrowDropdown'
import Action from '../Action'

export default {
  components: {
    Action,
    Dropdown,
    BaseInput,
    IconArrowDropdown
  },
  model: {
    event: 'change'
  },
  props: {
    value: { type: null, default: null },
    options: { type: Array, required: true },
    placeholder: { type: String, default: 'Select...' }
  },
  computed: {
    valueLabel() {
      for (const option of this.options) {
        if (option[0] === this.value) return option[1]
      }

      return null
    }
  },
  methods: {
    selectOption(option, close) {
      this.$emit('change', option[0])
      close()
    }
  }
}
</script>

<style lang="scss">
@import "../../../assets/styles/_variables.scss";

.base-select {
  .base-input {
    width: 100%;
  }

  &.dropdown_opened {
    .base-select__input-wrap .icon_arrow-dropdown {
      color: $color-text-placeholder
    }
  }
}

.base-select__input-wrap {
  position: relative;

  .icon_arrow-dropdown {
    position: absolute;
    top: 16px;
    right: 13px;
    color: $color-text-tertiary;
    transition: color $hover-transition-speed;
  }
}

</style>
