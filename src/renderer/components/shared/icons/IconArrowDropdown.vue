<template>
  <svg v-if="thin" width="9" height="15" xmlns="http://www.w3.org/2000/svg" :class="className">
    <path
      d="M.68 1.486a.611.611 0 0 1 0-.9.703.703 0 0 1 .954 0L8.22 6.8a.815.815 0 0 1 0 1.2l-6.585 6.214a.703.703 0 0 1-.954 0 .611.611 0 0 1 0-.9L6.948 7.4.68 1.486z"
      fill="currentColor"
      fill-rule="nonzero"
    />
  </svg>

  <svg v-else width="15" height="8" xmlns="http://www.w3.org/2000/svg" :class="className">
    <path
      d="M1.79.296a.993.993 0 0 0-1.414 0 1.018 1.018 0 0 0 0 1.429l5.858 5.92a1.191 1.191 0 0 0 1.697 0l5.859-5.92c.39-.395.39-1.034 0-1.429a.993.993 0 0 0-1.414 0L7.083 5.644 1.79.296z"
      fill="currentColor"
      fill-rule="nonzero"
    />
  </svg>
</template>

<script>
const TOS = ['top', 'bottom', 'left', 'right']

export default {
  props: {
    to: { type: String, default: 'bottom', validator: val => TOS.includes(val) },
    thin: { type: Boolean, default: false }
  },
  computed: {
    className() {
      return {
        'icon': true,
        'icon_arrow-dropdown': true,
        'icon_thin': this.thin,
        [`icon_to_${this.to}`]: true
      }
    }
  }
}
</script>

<style lang="scss">
.button > .icon_arrow-dropdown {
  margin: 0 -5px 1px 10px;
}

.icon_arrow-dropdown {
  &.icon_to_top {
    transform: rotate(180deg);

    &.icon_thin {
      transform: rotate(-90deg);
    }
  }

  &.icon_to_right {
    transform: rotate(-90deg);

    &.icon_thin {
      transform: none;
    }
  }
}
</style>
