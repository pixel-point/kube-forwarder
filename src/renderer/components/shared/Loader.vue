<template>
  <div class="loader" />
</template>

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
    margin: -15px 0 0 -15px;
    left: 50%;
    width: 30px;
    height: 30px;
    border: 1px solid $color-primary;
    border-radius: 50%;
    border-bottom-color: transparent !important;
    animation: spin 1s linear infinite;
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
