<template>
  <div class="logo">
    <div class="logo__square logo__square_top" />
    <div class="logo__square logo__square_right" />
    <div class="logo__square logo__square_bottom" />
    <div class="logo__square logo__square_left" />
  </div>
</template>

<style lang="scss">
@import "../../assets/styles/_variables.scss";

.logo {
  position: relative;
  width: 36px;
  height: 36px;
}

.logo__square {
  width: 12px;
  height: 12px;
  border-radius: 1px;
  position: absolute;
  transform: rotate(45deg);
  display: inline-block;
}

.logo__square_top,
.logo__square_bottom {
  background-color: $color-primary
}

.logo__square_right,
.logo__square_left {
  background-color: $color-secondary
}

.logo__square_top {
  top: 2px;
  left: 12px;
}

.logo__square_right {
  top: 12px;
  right: 2px;
}

.logo__square_bottom {
  bottom: 2px;
  left: 12px;
}

.logo__square_left {
  top: 12px;
  left: 2px;
}
</style>
