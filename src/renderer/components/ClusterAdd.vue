<template>
  <div class="page clusters-add">
    <Header :back-path="backPath" />
    <div class="page__block clusters-add__configs-block">
      <template v-if="configs.length">
        <span v-if="filesOpened">
          We have detected the following clusters in the submitted config.
          Please choose clusters you want to add:
        </span>
        <span v-else>We have detected existing <b>~/.kube/config</b> file with the following clusters:</span>
        <div class="clusters-add__configs">
          <div v-for="config in configs" :key="config.filePath">
            <div v-if="configs.length > 1" class="clusters-add__file-path">{{ config.filePath }}</div>

            <div v-for="context in config.contexts" :key="context.name" class="clusters-add__context">
              <BaseCheckbox v-model="config.checkedContextsIndex[context.name]">
                <b>{{ context.cluster }}</b>
                <span v-if="config.nonUniqClusters[context.cluster]">(user: <b>{{ context.user }}</b>)</span>
              </BaseCheckbox>
            </div>
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

    <div class="page__blocks-row">
      <div class="page__block clusters-add__option">
        <div>Add config manually</div>
        <Button layout="outline" theme="primary" @click="handleOpenFile">OPEN A FILE(S)</Button>
        or
        <Button layout="outline" theme="primary" to="/clusters/new">USING A FORM</Button>
      </div>

      <div class="page__block clusters-add__option">
        <div>Restore from <b>Kube Forwarder JSON</b></div>
        <Button layout="outline" theme="primary" to="/clusters/import?back=/clusters/add">RESTORE FROM JSON</Button>
      </div>
    </div>
  </div>
</template>

<script>
import { remote } from 'electron'
import * as path from 'path'
import { KubeConfig } from '@kubernetes/client-node'
import { mapActions } from 'vuex'
import * as Sentry from '@sentry/electron'

import Header from './shared/Header'
import Button from './shared/Button'
import BaseCheckbox from './shared/form/BaseCheckbox'
import { checkConnection } from '../lib/helpers/cluster'
import { showConfirmBox, showOpenDialog } from '../lib/helpers/ui'
import * as configStoringMethods from '../lib/constants/config-storing-methods'

const { app } = remote

export default {
  components: {
    BaseCheckbox,
    Header,
    Button
  },
  data() {
    return {
      filesOpened: false,
      saving: false,
      configs: [
        // contexts,
        // filePath,
        // checkedContextsIndex,
        // nonUniqClusters
      ]
    }
  },
  computed: {
    backPath() {
      return '/'
    },
    isAnySelected() {
      return Boolean(this.configs.find(x => Object.values(x.checkedContextsIndex).includes(true)))
    }
  },
  async mounted() {
    const kubeConfigDefaultPath = path.join(app.getPath('home'), '.kube/config')
    try {
      this.addConfig(kubeConfigDefaultPath)
    } catch (error) {
      console.log(error)
    }
  },
  methods: {
    ...mapActions('Clusters', ['createCluster']),
    addConfig(filePath) {
      const kubeConfig = new KubeConfig()
      kubeConfig.loadFromFile(filePath)

      const contexts = kubeConfig.contexts
      const checkedContextsIndex = {}

      for (const context of contexts) {
        checkedContextsIndex[context.name] = false
      }

      this.configs.push({
        filePath,
        kubeConfig,
        contexts,
        checkedContextsIndex,
        nonUniqClusters: this.buildNonUniqClusters(contexts)
      })
    },
    // buildNonUniqClusters returns the index of context.names whos clusters occur al least twice.
    buildNonUniqClusters(contexts) {
      const index = {}
      const result = {}
      for (const context of contexts) {
        if (index[context.cluster]) {
          result[context.cluster] = true
        } else {
          index[context.cluster] = true
        }
      }

      return result
    },
    async handleSave() {
      if (this.saving) return
      this.saving = true

      const selectedPairs = []
      for (const config of this.configs) {
        for (const context of config.contexts) {
          if (config.checkedContextsIndex[context.name]) {
            selectedPairs.push({ config, context })
          }
        }
      }

      const connectionResult = await this.checkConnectionToPairs(selectedPairs)
      const continueSaving = connectionResult.success || await this.confirmInvalidConnection(connectionResult.errors)

      if (continueSaving) {
        this.savePairs(selectedPairs)
        this.$router.push('/')
      }

      this.saving = false
    },
    savePairs(pairs) {
      for (const { config, context } of pairs) {
        const name = config.nonUniqClusters[context.cluster] ? `${context.cluster} — ${context.user}` : context.cluster

        this.createCluster({
          name,
          config: {
            storingMethod: configStoringMethods.PATH,
            path: config.filePath,
            currentContext: context.name
          }
        })
      }
    },
    async checkConnectionToPairs(pairs) {
      const errors = []

      for (const { config, context } of pairs) {
        const error = await checkConnection(config.kubeConfig, context.name)
        if (error) errors.push({ error, contextName: context.name })
      }

      return { success: errors.length === 0, errors }
    },
    async confirmInvalidConnection(errors) {
      const messages = errors.map(({ error, contextName }) => {
        // TODO a breadcrumb for originError
        Sentry.captureException(error)
        return this.getConnectionErrorMessage(error, contextName)
      })

      const message = messages.concat(['Do you want to continue saving?']).join('\n\n')
      return showConfirmBox(message)
    },
    getConnectionErrorMessage(error, contextName) {
      const { originError } = error

      if (originError && typeof originError.message === 'string') {
        const awsNotFoundMatch = originError.message.match(/\s(aws: command not found)/)
        if (awsNotFoundMatch) {
          return `Failed to connect to ${contextName}: ${awsNotFoundMatch[1]}. ` +
            'Please make sure you have installed AWS CLI. (https://docs.aws.amazon.com/cli/)'
        }

        const gcloudNotFoundMatch = originError.message.match(/\W(gcloud: No such file or directory)/)
        if (gcloudNotFoundMatch) {
          return `Failed to connect to ${contextName}: ${gcloudNotFoundMatch[1]}. ` +
            'Please make sure you have installed Google Cloud SDK. (https://cloud.google.com/sdk)'
        }
      }

      const originMessagePart = `${error.originError.message ? `, ${error.originError.message}` : ''}`
      return `Failed to connect to ${contextName}: ${error.message}${originMessagePart}`
    },
    async handleOpenFile() {
      const filePaths = await showOpenDialog({ properties: ['openFile', 'multiSelections'] })

      if (filePaths) {
        this.filesOpened = true
        this.configs = []
        for (const filePath of filePaths) {
          this.addConfig(filePath)
        }
      }
    }
  }
}

</script>

<style lang="scss">
.clusters-add__configs-block {
  margin-bottom: 20px;
}

.clusters-add__configs {
  margin: 30px 0;
}

.clusters-add__file-path {
  margin: 30px 0 10px;
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

.clusters-add__option {
  .button {
    margin-top: 30px
  }
}
</style>
