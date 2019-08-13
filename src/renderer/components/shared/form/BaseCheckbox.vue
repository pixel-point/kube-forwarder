<template>
  <div class="base-checkbox">
    <input
      :id="id"
      class="base-checkbox__input"
      type="checkbox"
      :checked="value"
      @change="$emit('input', $event.target.checked)"
    >
    <label class="base-checkbox__label" :for="id">
      <slot />
    </label>
  </div>
</template>

<script>
export default {
  name: 'BaseCheckbox',
  props: {
    name: { type: String, default: () => Math.random().toString() },
    value: { type: Boolean, default: null }
  },
  computed: {
    id() {
      return `base-checkbox-${this.name}`
    }
  }
}
</script>

<!-- TODO: Think about extracting common code from BaseCheckbox and BaseRadioButtons -->

<style lang="scss">
.base-checkbox {
  display: inline-block;
}

.base-checkbox__input {
  width: 0;
  height: 0;
  opacity: 0;
  position: absolute;
}

.base-checkbox__input:focus + .base-checkbox__label:before {
  box-shadow: 0 0 4px 1px #0564d7;
}

.base-checkbox__input:checked + .base-checkbox__label:before {
  background: #0564d7;
  border-color: transparent;
}

.base-checkbox__input:checked + .base-checkbox__label:after {
  content: "";
  position: absolute;
  left: 6px;
  top: 6px;
  width: 9px;
  height: 6px;
  border: solid white;
  border-width: 0 0 2px 2px;
  transform: rotate(-45deg);
  box-sizing: border-box;
}

.base-checkbox__label {
  position: relative;
  padding-left: 27px;
  line-height: 20px;
  display: inline-block;
  cursor: pointer;

  &:before {
    content: "";
    position: absolute;
    width: 18px;
    height: 18px;
    border: 1px solid rgba(25, 45, 70, 0.25);
    border-radius: 3px;
    top: 0;
    left: 0;
    background-color: #fff;
  }

  &:hover {
    &:before {
      border-color: #0564d7;
    }
  }
}
</style>
