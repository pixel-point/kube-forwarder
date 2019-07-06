<template>
  <div :class="className" />
</template>

<script>
export default {
  name: 'Loader',
  props: {
    size: { type: String, default: 'm' }
  },
  computed: {
    className() {
      return {
        loader: true,
        [`loader_size_${this.size}`]: true
      }
    }
  }
}
</script>

<style lang="scss">
@import '../../assets/styles/variables';

.loader {
  width: 100%;
  height: 100%;
  position: relative;

  &:before {
    content: "";
    display: inline-block;
    position: absolute;
    top: 50%;
    left: 50%;
    border: 1px solid $color-primary;
    border-radius: 50%;
    border-bottom-color: transparent !important;
    animation: spin 1s linear infinite;
    box-sizing: border-box;
  }

  &.loader_size_s:before {
    width: 20px;
    height: 20px;
    margin: -10px 0 0 -10px;
  }

  &.loader_size_m:before {
    width: 30px;
    height: 30px;
    margin: -15px 0 0 -15px;
  }
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
