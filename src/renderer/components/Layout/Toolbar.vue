<template>
  <div class="toolbar">
    <template v-if="isWebDemo">
      <img src="../../assets/images/macos-system-buttons.svg" alt="system buttons" />
      <div class="space" />
      <div class="toolbar__demo-label"> Demonstration Version </div>
    </template>
    <template v-else>
      <div class="space" />
      <div class="toolbar__version" @click="showVerbose">v{{ version }}</div>
    </template>
  </div>
</template>

<script>
import packageJs from '../../../../package'
import { showMessageBox } from '../../lib/helpers/ui'
import { isWebDemo } from '../../lib/environment'

export default {
  name: 'Toolbar',
  computed: {
    version: function() {
      return packageJs.versionString
    },
    isWebDemo() {
      return isWebDemo
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

.toolbar__demo-label {
  padding: 4px 20px;
  font-weight: 600;
  font-size: 13px;
  line-height: 16px;
  color: #fff;
  background: #3273E1;
  background: linear-gradient(72.1deg, #3273E1 0%, #1ECDE1 100%);
  text-shadow: 0 1px 1px rgba(20, 45, 85, 0.25);
  border-radius: 2px;
}

.toolbar__version {
  color: $color-text-quaternary;
  float: right;
  flex-shrink: 0;
}
</style>
