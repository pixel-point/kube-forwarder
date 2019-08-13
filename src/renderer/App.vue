<template>
  <router-view />
</template>

<script>
export default {
  name: 'KubernetesPortForwarder',
  mounted() {
    if (process.env.IS_WEB) {
      document.body.classList.add('body_target_web')
    }
    if (process.env.ENV === 'test') {
      document.body.classList.add('body_env_test')
    }
  }
}
</script>

<style lang="scss">
@import "assets/styles/icon";
@import "assets/styles/table";
@import "assets/styles/text";
@import "assets/styles/page";

// Note: For new web sites, it is often useful to begin by setting box-sizing to border-box.
// https://developer.mozilla.org/en-US/docs/Web/CSS/box-sizing
* {
  box-sizing: border-box;
}

html {
  height: 100%;
}

body {
  margin: 0;
  min-height: 100%;
  width: 100%;
  font-family: system-ui, serif;
  font-size: 14px;
  color: #142d55; /* = $color-text */
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  display: flex;
}

.body_target_web {
  font-family: Helvetica !important;
}

td, th {
  padding: 0
}

.space {
  flex-grow: 1;
}

b {
  font-weight: 600;
}

.link {
  text-decoration: none;
  color: $color-primary;
  position: relative;

  &:after {
    content: '';
    position: absolute;
    bottom: -2px;
    left: 0;
    right: 0;
    height: 1px;
    background-color: rgba($color-primary, 0.5);
  }
}
</style>
