<template>
  <div class="page page-cluster-new">
    <Header :back-path="backPath" />

    <BaseForm class="cluster-form" @submit="handleSubmit">
      <ControlGroup
        label="Config file"
        hint="Get this from ~/.kube/config or your cloud provider"
        :attribute="$v.config"
      >
        <BaseTextArea v-model.trim="$v.config.$model" />
      </ControlGroup>

      <div class="control-actions">
        <Button theme="danger" layout="outline" :to="backPath">Cancel</Button>
        <div class="space" />
        <div v-if="error" class="control-actions__error">{{ error }}</div>
        <Button type="submit" theme="primary" :disabled="$v.$invalid">Next</Button>
      </div>
    </BaseForm>
  </div>
</template>

<script>
import { validationMixin } from 'vuelidate'
import { required } from 'vuelidate/lib/validators'
import { KubeConfig } from '@kubernetes/client-node'

import Header from './shared/Header'
import BaseForm from './shared/form/BaseForm'
import BaseTextArea from './shared/form/BaseTextArea'
import Button from './shared/Button'
import ControlGroup from './shared/form/ControlGroup'

export default {
  components: {
    Header,
    BaseForm,
    ControlGroup,
    BaseTextArea,
    Button
  },
  mixins: [validationMixin],
  data() {
    return {
      config: null,
      error: null
    }
  },
  validations: {
    config: { required }
  },
  computed: {
    backPath() {
      return this.$route.query.back || '/'
    }
  },
  methods: {
    handleSubmit() {
      this.$v.$touch()
      if (this.$v.$invalid) return

      try {
        new KubeConfig().loadFromString(this.config)
      } catch (error) {
        this.error = `The config is invalid: ${error.message}`
        return
      }

      this.$store.state.manualClusterConfig = this.config
      this.$router.push(this.backPath)
    }
  }
}

</script>
