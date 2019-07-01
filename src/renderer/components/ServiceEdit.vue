<template>
  <div class="page">
    <Header :back-path="backPath" />
    <ServiceForm :service-id="serviceId" :initial-attributes="initialAttributes" />
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
    serviceId() {
      return this.$route.params.id
    },
    initialAttributes() {
      return this.$store.state.Services.items[this.serviceId]
    }
  }
}
</script>
