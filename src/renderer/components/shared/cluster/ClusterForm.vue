<template>
  <BaseForm class="cluster-form" @submit="handleSubmit">
    <ControlGroup label="Cluster name" :attribute="$v.attributes.name">
      <BaseInput v-model.trim="$v.attributes.name.$model" />
    </ControlGroup>

    <ControlGroup :attribute="$v.attributes.config.storingMethod">
      <RadioButtonGroup v-model="$v.attributes.config.storingMethod.$model"
                        theme="primary"
                        :options="configStoringMethodsOptions" />
    </ControlGroup>

    <ControlGroup>
      <Button theme="primary" size="s" @click="handleOpenFile">Open a file</Button>
    </ControlGroup>

    <ControlGroup
      v-if="attributes.config.storingMethod === configStoringMethods.CONTENT"
      label="Config file"
      hint="Get this from ~/.kube/config or your cloud provider"
      :attribute="$v.attributes.config.content"
    >
      <BaseTextArea v-model.trim="$v.attributes.config.content.$model" />
    </ControlGroup>
    <ControlGroup
      v-if="attributes.config.storingMethod === configStoringMethods.PATH"
      label="Path"
      :attribute="$v.attributes.config.path"
    >
      <BaseInput v-model.trim="$v.attributes.config.path.$model" />
    </ControlGroup>

    <ControlGroup label="Current context" :attribute="$v.attributes.config.currentContext">
      <BaseInput v-model.trim="$v.attributes.config.currentContext.$model" />
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

import BaseForm from '../form/BaseForm'
import BaseInput from '../form/BaseInput'
import BaseTextArea from '../form/BaseTextArea'
import Button from '../Button'
import ControlGroup from '../form/ControlGroup'
import { checkConnection, buildKubeConfig } from '../../../lib/helpers/cluster'
import { showMessageBox, showOpenDialog } from '../../../lib/helpers/ui'
import RadioButtonGroup from '../RadioButtonGroup'
import * as configStoringMethods from '../../../lib/constants/config-storing-methods'

export default {
  name: 'ClusterForm',
  components: {
    RadioButtonGroup,
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
      checkingConnection: false,
      attributes: {
        ...this.buildCluster(),
        clusterId: this.clusterId,
        ...cloneDeep(this.initialAttributes)
      }
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
        [configStoringMethods.PATH, 'Path'],
        [configStoringMethods.CONTENT, 'Content']
      ]
    },
    configStoringMethods: () => configStoringMethods
  },
  methods: {
    ...mapActions('Clusters', ['createCluster', 'updateCluster']),
    buildCluster() {
      return {
        name: '',
        config: { storingMethod: configStoringMethods.PATH, path: '', content: '', currentContext: '' }
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
    async handleOpenFile() {
      const filePaths = await showOpenDialog({ properties: ['openFile'] })
      if (!filePaths) return

      if (this.attributes.config.storingMethod === configStoringMethods.PATH) {
        this.attributes.config.path = filePaths[0]
      } else if (this.attributes.config.storingMethod === configStoringMethods.CONTENT) {
        this.attributes.config.content = await fs.readFile(filePaths[0], { encoding: 'utf8' })
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
}
</style>
