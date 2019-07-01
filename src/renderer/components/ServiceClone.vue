<template>
  <div class="page">
    <Header :back-path="backPath" />
    <ServiceForm :initial-attributes="initialAttributes" />
  </div>
</template>

<script>
import ServiceForm from './shared/service/ServiceForm'
import Header from './shared/Header'

export default {
  components: {
    ServiceForm,
    Header
  },
  computed: {
    backPath() {
      return '/'
    },
    initialAttributes() {
      const serviceId = this.$route.params.id
      return { ...this.$store.state.Services.items[serviceId], name: '' }
    }
  }
}
</script>
