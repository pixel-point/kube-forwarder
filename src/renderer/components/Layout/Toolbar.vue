<template>
  <div class="toolbar">
    <div class="space" />
    <div class="toolbar__version" @click="showVerbose">v{{ version }}</div>
  </div>
</template>

<script>
import packageJs from '../../../../package'
import { showMessageBox } from '../../lib/helpers/ui'

export default {
  name: 'Toolbar',
  computed: {
    version: function() {
      return packageJs.versionString
    }
  },
  methods: {
    showVerbose() {
      const message = [
        `Version label: ${this.version}`,
        `Version: ${packageJs.version}`,
        `Build: ${process.env.BUILD}`,
        `Env: ${process.env.NODE_ENV}`
      ].join('\n')
      showMessageBox(message)
    }
  }
}

</script>

<style lang="scss">
@import "../../assets/styles/_variables.scss";

.toolbar {
  -webkit-app-region: drag;
  height: $toolbar-height;
  border-bottom: $border;
  display: flex;
  align-items: center;
  padding: 0 20px;
  position: fixed;
  right: 0;
  left: 0;
  top: 0;
  z-index: 100;
  background: #fff;
}

.toolbar__version {
  color: $color-text-quaternary;
  float: right;
  flex-shrink: 0;
}
</style>
