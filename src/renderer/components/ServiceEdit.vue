<template>
  <ServiceForm
    :initial-attributes="initialAttributes"
    :edit-id="serviceId"
  />
</template>

<script>
import ServiceForm from '@/components/ServiceForm'

export default {
  components: {
    ServiceForm
  },
  computed: {
    serviceId() {
      return this.$route.params.id
    },
    initialAttributes() {
      return this.$store.state.Services.items[this.serviceId]
    }
  }
}
</script>
