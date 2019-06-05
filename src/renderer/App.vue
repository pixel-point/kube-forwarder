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
  }
}
</script>

<style lang="scss">
@import "assets/styles/icon";
@import "assets/styles/table";

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

table {
  border-collapse: collapse;
}

.space {
  flex-grow: 1;
}
</style>
