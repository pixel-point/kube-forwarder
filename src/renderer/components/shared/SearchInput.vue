<template>
  <div class="search-input">
    <BaseInput v-bind="$attrs" :placeholder="placeholder" type="search" v-on="$listeners" />
    <IconMagnifier />
  </div>
</template>

<script>
import BaseInput from './form/BaseInput'
import IconMagnifier from './icons/IconMagnifier'

export default {
  name: 'SearchInput',
  components: {
    BaseInput,
    IconMagnifier
  },
  props: {
    placeholder: { type: String, default: 'Search...' }
  }
}
</script>

<!-- eslint-disable max-len -->
<style lang="scss">
@import "../../assets/styles/variables";

.search-input {
  position: relative;

  .icon_magnifier {
    position: absolute;
    top: 10px;
    left: 13px;
    color: $color-text-quaternary;
    pointer-events: none;
  }

  .icon_cross {
    position: absolute;
    top: 13px;
    right: 13px;
    color: $color-text-quaternary;
  }

  .base-input {
    padding-left: 43px;
    width: 100%;

    &::-webkit-search-cancel-button {
      -webkit-appearance: none;
      width: 10px;
      height: 10px;
      cursor: pointer;
      opacity: 0.25;
      background-image: url("data:image/svg+xml,%3Csvg width='10' height='11' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M6.374 5.404l3.372 3.372a.954.954 0 1 1-1.349 1.348L5.025 6.753l-3.372 3.371A.954.954 0 1 1 .304 8.776l3.372-3.372L.304 2.032A.954.954 0 0 1 1.654.683l3.371 3.372L8.397.683a.954.954 0 0 1 1.349 1.349L6.374 5.404z' fill='%23142d55' fill-rule='nonzero'%0Aopacity='1' /%3E%3C/svg%3E");
      transition: opacity $hover-transition-speed;

      &:hover {
        opacity: 0.75;
      }
    }

    &:focus + .icon_magnifier {
      color: $color-text-secondary;
    }
  }
}
</style>
<!-- eslint-enable max-len -->
