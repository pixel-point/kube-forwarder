<template>
  <ServiceForm :initial-attributes="initialAttributes" />
</template>

<script>
import ServiceForm from '@/components/ServiceForm'

export default {
  components: {
    ServiceForm
  },
  computed: {
    initialAttributes() {
      const serviceId = this.$route.params.id
      return { ...this.$store.state.Services.items[serviceId], name: '' }
    }
  }
}
</script>
