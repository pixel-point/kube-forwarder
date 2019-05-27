<template>
  <router-view />
</template>

<script>
export default {
  name: 'KubernetesPortForwarder'
}
</script>

<style lang="scss">
@import "assets/styles/icon";
@import "assets/styles/table";

html, body {
  margin: 0;
  min-height: 100%;
  width: 100%;
}

html {
  display: flex;
  font-family: system-ui, serif;
  font-size: 14px;
  color: #142d55; /* = $color-text */
}

td, th {
  padding: 0
}

table {
  border-collapse: collapse;
}

* {
  box-sizing: border-box;
}

.space {
  flex-grow: 1;
}
</style>
