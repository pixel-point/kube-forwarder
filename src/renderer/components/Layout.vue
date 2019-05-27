<template>
  <div class="app-container">
    <Toolbar />

    <main>
      <router-view />
    </main>
  </div>
</template>

<script>
import Toolbar from '@/components/Layout/Toolbar'

export default {
  name: 'Layout',
  components: {
    Toolbar
  }
}

</script>

<style lang="scss">
@import "../assets/styles/_variables.scss";

main {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  padding-top: $toolbar-height
}

.app-container {
  min-height: 100%;
  display: flex;
  flex-direction: column;
}
</style>
