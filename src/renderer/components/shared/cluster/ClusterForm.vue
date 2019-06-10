<template>
  <BaseForm class="cluster-form" @submit="handleSubmit">
    <ControlGroup label="Cluster name" :attribute="$v.attributes.name">
      <BaseInput v-model.trim="$v.attributes.name.$model" />
    </ControlGroup>

    <ControlGroup
      label="Config file"
      hint="Get this from ~/.kube/config or your cloud provider"
      :attribute="$v.attributes.config"
    >
      <BaseTextArea v-model.trim="$v.attributes.config.$model" />
    </ControlGroup>

    <div class="control-actions">
      <Button theme="danger" layout="outline" :to="backPath">Cancel</Button>
      <div class="space" />
      <div v-if="error" class="control-actions__error">{{ error }}</div>
      <div v-else class="control-actions__message">{{ message }}</div>
      <Button type="submit" theme="primary" :disabled="$v.$invalid">{{ submitButtonTitle }}</Button>
    </div>
  </BaseForm>
</template>

<script>
import cloneDeep from 'clone-deep'
import { mapActions } from 'vuex'
import { required } from 'vuelidate/lib/validators'
import { validationMixin } from 'vuelidate'

import BaseForm from '../form/BaseForm'
import BaseInput from '../form/BaseInput'
import BaseTextArea from '../form/BaseTextArea'
import Button from '../Button'
import ControlGroup from '../form/ControlGroup'

export default {
  name: 'ClusterForm',
  components: {
    BaseInput,
    BaseForm,
    BaseTextArea,
    Button,
    ControlGroup
  },
  mixins: [validationMixin],
  props: {
    clusterId: { type: String, default: null },
    initialAttributes: { type: Object, default: () => ({}) },
    message: { type: String, default: '' }
  },
  data() {
    return {
      error: null,
      attributes: {
        ...this.buildCluster(),
        clusterId: this.clusterId,
        ...cloneDeep(this.initialAttributes)
      }
    }
  },
  validations: {
    attributes: {
      name: { required },
      config: { required }
    }
  },
  computed: {
    submitButtonTitle() {
      return this.clusterId ? `Save` : 'Add a cluster'
    },
    backPath() {
      return '/'
    }
  },
  methods: {
    ...mapActions('Clusters', ['createCluster', 'updateCluster']),
    buildCluster() {
      return { name: '', config: '' }
    },
    async handleSubmit() {
      this.$v.$touch()

      if (this.$v.$invalid) {
        this.error = 'Please, check your configuration, something went wrong'
        return
      }

      const action = this.clusterId ? 'updateCluster' : 'createCluster'
      const result = await this[action](this.attributes)

      if (result.success) {
        if (this.$listeners.success) this.$listeners.success(result.item)
        else this.$router.push('/')
      } else {
        this.error = result.errors[0].toString()
      }
    }
  }
}
</script>

<style lang="scss">
.cluster-form {
  .controls > .base-textarea {
    height: 193px;
    resize: none;
  }
}
</style>
