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
            <BaseCheckbox v-model="checkedContexts[context.name]">
              <b>{{ context.cluster }}</b>
              <span v-if="nonUniqClusters[context.cluster]">(user: <b>{{ context.user }}</b>)</span>
            </BaseCheckbox>
            <Button layout="outline"
                    theme="primary"
                    size="s"
                    :loading="checkingConnection === context.name"
                    :disabled="checkingConnection && checkingConnection !== context.name"
                    @click="handleCheckConnection(context.name)"
            >
              Check connection
            </Button>
          </div>
        </div>

        <div class="clusters-add__actions">
          <Button layout="filled" theme="primary" :disabled="!isAnySelected" @click="saveSelected">
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
      checkedContexts: {},
      checkingConnection: false
    }
  },
  computed: {
    backPath() {
      return '/'
    },
    isAnySelected() {
      return Boolean(Object.values(this.checkedContexts).find(x => x))
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
      Vue.set(this.checkedContexts, context.name, false)
    }
  },
  methods: {
    ...mapActions('Clusters', ['createCluster']),
    async getConfigString() {
      if (this.manualConfig) return this.manualConfig

      const kubeConfigDefaultPath = path.join(app.getPath('home'), '.kube/config')
      return fs.readFile(kubeConfigDefaultPath, { encoding: 'utf8' }).catch(() => '')
    },
    saveSelected() {
      for (const context of this.contexts) {
        if (!this.checkedContexts[context.name]) continue

        this.configObject['current-context'] = context.name
        const name = this.nonUniqClusters[context.cluster] ? `${context.cluster} — ${context.user}` : context.cluster
        this.createCluster({ name, config: yaml.safeDump(this.configObject) })
      }

      this.$router.push('/')
    },
    async handleCheckConnection(contextName) {
      if (!this.checkingConnection && this.kubeConfig && typeof this.kubeConfig.makeApiClient === 'function') {
        this.checkingConnection = contextName
        await checkConnection(this.kubeConfig, contextName)
        this.checkingConnection = false
      }
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
