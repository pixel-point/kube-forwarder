<template>
  <div
    class="alert"
    :class="{ [`alert_theme_${theme}`]: true }"
  >
    <slot />
  </div>
</template>

<script>
export default {
  props: {
    theme: { type: String, default: 'info' }
  }
}
</script>

<style scoped lang="scss">
@import "../../assets/styles/variables";

.alert {
  text-align: center;
  padding: 10px 15px;
  border-radius: 3px;
}

.alert_theme_info,
.alert_theme_warning,
.alert_theme_danger,
.alert_theme_success {
  color: #fff;
  font-weight: 600;
  line-height: normal;
}

.alert_theme_success {
  background-color: $color-success;
}

.alert_theme_info {
  background-color: $color-info;
}

.alert_theme_warning {
  background-color: $color-warning;
}

.alert_theme_danger {
  background-color: $color-danger;
}
</style>
