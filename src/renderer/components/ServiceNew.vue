<template>
  <div class="page page_name_service-new">
    <Header :back-path="backPath" />
    <Alert v-if="alert" v-bind="alert">{{ alert.message }}</Alert>
    <ServiceForm />
  </div>
</template>

<script>
import Header from './shared/Header'
import ServiceForm from './shared/service/ServiceForm'
import Alert from './shared/Alert'

export default {
  components: {
    ServiceForm,
    Header,
    Alert
  },
  computed: {
    backPath() {
      return '/'
    },
    alert() {
      if (this.$route.query.messageKey === 'clusterCreated') {
        return {
          theme: 'success',
          message: 'We successfully added your cluster. Add your first service by filling the form below.'
        }
      }

      return null
    }
  }
}
</script>

<style lang="scss">
</style>
