<template>
  <div class="header">
    <router-link v-if="backPath" :to="backPath">
      <Arrow class="header__back" />
    </router-link>

    <slot />

    <router-link to="/">
      <Logo />
    </router-link>
  </div>
</template>

<script>
import Logo from './Logo'
import Arrow from './icons/IconArrow'

export default {
  name: 'Header',
  components: {
    Logo,
    Arrow
  },
  props: {
    backPath: { type: String, default: null }
  }
}
</script>

<style lang="scss">
@import "../../assets/styles/variables";

.header {
  height: 36px;
  position: relative;
  display: flex;
  align-items: center;

  .logo {
    position: absolute;
    left: 50%;
    top: 0;
    margin-left: -18px
  }
}

.header__back {
  cursor: pointer;
  color: $color-text-quaternary;
  transition: color $hover-transition-speed;

  &:hover {
    color: $color-text-secondary;
  }
}
</style>
