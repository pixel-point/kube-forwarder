<template>
  <div class="page clusters-add">
    <Header :back-path="backPath" />
    <div class="page__block clusters-add__contexts-block">
      <template v-if="contexts.length">
        <span v-if="manualConfig">
          We have detected the following clusters in the submitted config.
          Please choose clusters you want to add:
        </span>
        <span v-else>We have detected existing <b>~/.kube/config</b> file with the following clusters:</span>
        <div class="clusters-add__contexts">
          <div v-for="context in contexts" :key="context.name" class="clusters-add__context">
            <BaseCheckbox v-model="checkedContextsIndex[context.name]">
              <b>{{ context.cluster }}</b>
              <span v-if="nonUniqClusters[context.cluster]">(user: <b>{{ context.user }}</b>)</span>
            </BaseCheckbox>
          </div>
        </div>

        <div class="clusters-add__actions">
          <Button layout="filled" theme="primary" :disabled="!isAnySelected" :loading="saving" @click="handleSave">
            ADD SELECTED CLUSTERS
          </Button>
        </div>
      </template>

      <template v-else>
        We haven't found a valid <b>~/.kube/config</b> file. Please choose one of options below.
      </template>
    </div>

    <div v-if="!manualConfig" class="page__blocks-row">
      <div class="page__block clusters-add__option">
        Add cluster manually by inserting <b>~/.kube/config</b>
        <Button layout="outline" theme="primary" to="/clusters/new?back=/clusters/add">ADD MANUALLY</Button>
      </div>

      <div class="page__block clusters-add__option">
        Restore from <b>Kube Forwarder JSON</b>
        <Button layout="outline" theme="primary" to="/clusters/import?back=/clusters/add">RESTORE FROM JSON</Button>
      </div>
    </div>
  </div>
</template>

<script>
import { remote } from 'electron'
import * as path from 'path'
import { promises as fs } from 'fs'
import { KubeConfig } from '@kubernetes/client-node'
import yaml from 'js-yaml'
import { mapActions } from 'vuex'
import Vue from 'vue'

import Header from './shared/Header'
import Button from './shared/Button'
import BaseCheckbox from './shared/form/BaseCheckbox'
import { checkConnection } from '../lib/helpers/cluster'
import { showConfirmBox } from '../lib/helpers/ui'

const { app } = remote

export default {
  components: {
    BaseCheckbox,
    Header,
    Button
  },
  data() {
    return {
      contexts: [],
      checkedContextsIndex: {},
      saving: false
    }
  },
  computed: {
    backPath() {
      return '/'
    },
    isAnySelected() {
      return this.selectedContexts.length > 0
    },
    // returns the index of context.names whos clusters occur al least twice.
    nonUniqClusters() {
      const index = {}
      const result = {}
      for (const context of this.contexts) {
        if (index[context.cluster]) {
          result[context.cluster] = true
        } else {
          index[context.cluster] = true
        }
      }

      return result
    },
    manualConfig() {
      return this.$store.state.manualClusterConfig
    },
    selectedContexts() {
      return this.contexts.filter(context => this.checkedContextsIndex[context.name])
    }
  },
  async mounted() {
    const configString = await this.getConfigString()

    const kubeConfig = new KubeConfig()
    let configObject
    try {
      kubeConfig.loadFromString(configString)
      configObject = yaml.safeLoad(configString)
    } catch (error) {
      return
    }

    this.kubeConfig = kubeConfig // Shouldn't be a part of the data.
    this.configObject = configObject
    this.contexts = kubeConfig.contexts
    for (const context of this.contexts) {
      Vue.set(this.checkedContextsIndex, context.name, false)
    }
  },
  methods: {
    ...mapActions('Clusters', ['createCluster']),
    async getConfigString() {
      if (this.manualConfig) return this.manualConfig

      const kubeConfigDefaultPath = path.join(app.getPath('home'), '.kube/config')
      return fs.readFile(kubeConfigDefaultPath, { encoding: 'utf8' }).catch(() => '')
    },
    async handleSave() {
      if (!this.kubeConfig) return
      if (this.saving) return
      this.saving = true

      const connectionResult = await this.checkConnectionToContexts(this.selectedContexts)
      const continueSaving = connectionResult.success || await this.confirmInvalidConnection(connectionResult.errors)

      if (continueSaving) {
        this.saveSelected()
        this.$router.push('/')
      }

      this.saving = false
    },
    saveSelected() {
      for (const context of this.selectedContexts) {
        this.configObject['current-context'] = context.name
        const name = this.nonUniqClusters[context.cluster] ? `${context.cluster} — ${context.user}` : context.cluster
        this.createCluster({ name, config: yaml.safeDump(this.configObject) })
      }
    },
    async checkConnectionToContexts(contexts) {
      const errors = []

      for (const context of contexts) {
        const error = await checkConnection(this.kubeConfig, context.name)
        if (error) errors.push({ error, contextName: context.name })
      }

      return { success: errors.length === 0, errors }
    },
    async confirmInvalidConnection(errors) {
      const messages = errors.map(this.getConnectionErrorMessage)
      const message = messages.concat(['Do you want to continue saving?']).join('\n\n')
      return showConfirmBox(message)
    },
    getConnectionErrorMessage({ error, contextName }) {
      const awsNotFoundMatch = error.details.match(/\s(aws: command not found)/)
      if (awsNotFoundMatch) {
        return `Failed to connect to ${contextName}: ${awsNotFoundMatch[1]}. ` +
          'Please make sure you have installed AWS CLI. (https://docs.aws.amazon.com/cli/)'
      }

      const gcloudNotFoundMatch = error.details.match(/\W(gcloud: No such file or directory)/)
      if (gcloudNotFoundMatch) {
        return `Failed to connect to ${contextName}: ${gcloudNotFoundMatch[1]}. ` +
          'Please make sure you have installed Google Cloud SDK. (https://cloud.google.com/sdk)'
      }

      console.error(error.parentError)

      return `Failed to connect to ${contextName}: ${error.details}`
    }
  }
}

</script>

<style lang="scss">
.clusters-add__contexts-block {
  margin-bottom: 20px;
}

.clusters-add__contexts {
  margin: 30px 0;
}

.clusters-add__option {
  .button {
    margin-top: 30px
  }
}

.clusters-add__context {
  display: flex;
  align-items: center;

  .button {
    margin-left: 20px;
  }

  & + .clusters-add__context {
    margin-top: 10px;
    word-break: break-all;
  }
}
</style>
