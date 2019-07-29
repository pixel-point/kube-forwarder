<template>
  <BaseForm class="cluster-form" @submit="handleSubmit">
    <ControlGroup label="Cluster name" :attribute="$v.attributes.name">
      <BaseInput v-model.trim="$v.attributes.name.$model" />
    </ControlGroup>

    <ControlGroup
      label="Set destination to your kube config or paste it as a text"
      :attribute="$v.attributes.config.storingMethod"
    >
      <BaseRadioButtons
        v-model="$v.attributes.config.storingMethod.$model"
        name="clusterStoringMethod"
        direction="column"
        :options="configStoringMethodsOptions"
      />
    </ControlGroup>

    <ControlGroup
      v-if="attributes.config.storingMethod === configStoringMethods.CONTENT"
      label="Config file"
      hint="Get this from ~/.kube/config or your cloud provider"
      :attribute="$v.attributes.config.content"
    >
      <Button theme="primary" size="s" @click="handleOpenFile(configStoringMethods.CONTENT)">Copy from a file</Button>
      <BaseTextArea v-model.trim="$v.attributes.config.content.$model" />
    </ControlGroup>

    <ControlGroup
      v-if="attributes.config.storingMethod === configStoringMethods.PATH"
      label="Path"
      :attribute="$v.attributes.config.path"
    >
      <Button theme="primary" size="s" @click="handleOpenFile(configStoringMethods.PATH)">Select a file</Button>
      <BaseInput v-model.trim="$v.attributes.config.path.$model" />
    </ControlGroup>

    <ControlGroup label="Current context" :attribute="$v.attributes.config.currentContext">
      <AutocompleteInput v-model.trim="$v.attributes.config.currentContext.$model" :options="contextOptions"/>
    </ControlGroup>

    <div class="control-actions">
      <Button theme="danger" layout="outline" :to="backPath">Cancel</Button>
      <div class="space" />
      <div v-if="error" class="control-actions__error">{{ error }}</div>
      <div v-else class="control-actions__message">{{ message }}</div>
      <Button layout="outline" theme="primary" :loading="checkingConnection" @click="handleCheckConnection">
        Check Connection
      </Button>
      <Button type="submit" theme="primary" :disabled="$v.$invalid">{{ submitButtonTitle }}</Button>
    </div>
  </BaseForm>
</template>

<script>
import cloneDeep from 'clone-deep'
import { mapActions } from 'vuex'
import { required } from 'vuelidate/lib/validators'
import { validationMixin } from 'vuelidate'
import { promises as fs } from 'fs'
import deepmerge from 'deepmerge'
import { KubeConfig } from '@kubernetes/client-node'

import BaseForm from '../form/BaseForm'
import BaseInput from '../form/BaseInput'
import AutocompleteInput from '../form/AutocompleteInput'
import BaseTextArea from '../form/BaseTextArea'
import Button from '../Button'
import ControlGroup from '../form/ControlGroup'
import { checkConnection, buildKubeConfig } from '../../../lib/helpers/cluster'
import { showMessageBox, showOpenDialog, showConfirmBox } from '../../../lib/helpers/ui'
import BaseRadioButtons from '../form/BaseRadioButtons'
import * as configStoringMethods from '../../../lib/constants/config-storing-methods'

export default {
  name: 'ClusterForm',
  components: {
    BaseRadioButtons,
    BaseInput,
    AutocompleteInput,
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
    // `deepmerge` is required to merge attributes.cluster.config
    return {
      error: null,
      checkingConnection: false,
      contexts: [],
      attributes: deepmerge({
        ...this.buildCluster(),
        clusterId: this.clusterId
      }, cloneDeep(this.initialAttributes))
    }
  },
  validations() {
    const config = {
      storingMethod: { required },
      currentContext: { required },
      path: {},
      content: {}
    }

    const storingMethod = this.attributes.config.storingMethod
    if (storingMethod === configStoringMethods.PATH) config.path.required = required
    if (storingMethod === configStoringMethods.CONTENT) config.content.required = required

    return {
      attributes: {
        name: { required },
        config
      }
    }
  },
  computed: {
    submitButtonTitle() {
      return this.isNew ? 'Add a cluster' : 'Save'
    },
    backPath() {
      return '/'
    },
    isNew() {
      return !this.clusterId
    },
    configStoringMethodsOptions() {
      return [
        [configStoringMethods.PATH, 'Set a path'],
        [configStoringMethods.CONTENT, 'Paste as a text']
      ]
    },
    configStoringMethods: () => configStoringMethods,
    contextOptions() {
      return this.contexts.map(x => x.name)
    }
  },
  methods: {
    ...mapActions('Clusters', ['createCluster', 'updateCluster']),
    buildCluster() {
      return {
        name: '',
        config: { storingMethod: configStoringMethods.CONTENT, path: '', content: '', currentContext: '' }
      }
    },
    async handleSubmit() {
      this.$v.$touch()

      if (this.$v.$invalid) {
        this.error = 'Please, check your configuration, something went wrong'
        return
      }

      return this.saveCluster()
    },
    async saveCluster() {
      const action = this.clusterId ? 'updateCluster' : 'createCluster'
      const result = await this[action](this.attributes)

      if (result.success) {
        if (this.$listeners.success) this.$listeners.success(result.item)
        else this.$router.push('/')
      } else {
        this.error = result.errors[0].toString()
      }
    },
    async handleCheckConnection() {
      if (this.checkingConnection) return

      let kubeConfig
      try {
        kubeConfig = buildKubeConfig(this.attributes.config)
      } catch (error) {
        showMessageBox('Config is invalid', { details: error.message })
        return
      }

      this.checkingConnection = true

      const error = await checkConnection(kubeConfig)
      if (error) {
        await showMessageBox('Connection failed', {
          details: `${error.message}${error.originError.message ? `\n${error.originError.message}` : ''}`
        })
      } else {
        await showMessageBox('Connection successful')
      }

      this.checkingConnection = false
    },
    async handleOpenFile(method) {
      const filePaths = await showOpenDialog({ properties: ['openFile'] })
      if (!filePaths) return

      const content = await fs.readFile(filePaths[0], { encoding: 'utf8' })
      let kubeConfig = new KubeConfig()
      try {
        kubeConfig.loadFromString(content)
      } catch (error) {
        kubeConfig = null
        const result = await showConfirmBox(
          `The files contains invalid config. \nError ${error.message}.\n\n Do you want to continue?`
        )
        if (!result) return
      }

      if (method === configStoringMethods.PATH) this.attributes.config.path = filePaths[0]
      if (method === configStoringMethods.CONTENT) this.attributes.config.content = content

      if (kubeConfig) {
        this.attributes.config.currentContext = kubeConfig.getCurrentContext()
        this.contexts = kubeConfig.contexts
      } else {
        this.contexts = []
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

  .control-actions {
    .button + .button {
      margin-left: 10px;
    }
  }

  .button + .base-textarea,
  .button + .base-input {
    margin-top: 4px;
  }
}
</style>
