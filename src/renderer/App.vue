<template>
  <router-view />
</template>

<script>
export default {
  name: 'KubernetesPortForwarder',
  computed: {
    theme() {
      return this.$store.state.Settings.theme
    }
  },
  watch: {
    theme(nextTheme, prevTheme) {
      document.body.classList.replace(`body_theme_${prevTheme}`, `body_theme_${nextTheme}`)
    }
  },
  mounted() {
    document.body.classList.add('body')
    document.body.classList.add(`body_theme_${this.theme}`)

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
@import "assets/styles/themes";

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
  color: var(--color-text);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  display: flex;
  background-color: var(--body-background-color);
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
</style>
