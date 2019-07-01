<template>
  <div class="page clusters-add">
    <Header :back-path="backPath" />
    <div class="page__block clusters-add__contexts-block">
      <template v-if="contexts.length">
        We have detected existing <b>~/.kube/config</b> file with the following clusters:
        <div class="clusters-add__contexts">
          <BaseCheckbox v-for="context in contexts" :key="context.name" v-model="checkedContexts[context.name]">
            <b>{{ context.name }}</b> (cluster: <b>{{ context.cluster }}</b>, user: <b>{{ context.user }}</b>)
          </BaseCheckbox>
        </div>

        <Button layout="filled" theme="primary" :disabled="!isAnySelected" @click="saveSelected">
          ADD SELECTED CLUSTERS
        </Button>
      </template>

      <template v-else>
        We haven't found a valid <b>~/.kube/config</b> file. Please choose one of options below.
      </template>
    </div>

    <div class="page__blocks-row">
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
      checkedContexts: {}
    }
  },
  computed: {
    backPath() {
      return '/'
    },
    isAnySelected() {
      return Boolean(Object.values(this.checkedContexts).find(x => x))
    }
  },
  async mounted() {
    const kubeConfigDefaultPath = path.join(app.getPath('home'), '.kube/config')

    const config = await fs.readFile(kubeConfigDefaultPath, { encoding: 'utf8' }).catch(() => null)
    if (!config) return

    const kubeConfig = new KubeConfig()
    let configObject
    try {
      kubeConfig.loadFromString(config)
      configObject = yaml.safeLoad(config)
    } catch (error) {
      return
    }

    this.configObject = configObject
    this.contexts = kubeConfig.contexts
    for (const context of this.contexts) {
      Vue.set(this.checkedContexts, context.name, false)
    }
  },
  methods: {
    ...mapActions('Clusters', ['createCluster']),
    saveSelected() {
      for (const context of this.contexts) {
        if (!this.checkedContexts[context.name]) return

        this.configObject['current-context'] = context.name
        this.createCluster({ name: context.name, config: yaml.safeDump(this.configObject) })
      }

      this.$router.push('/')
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

  .base-checkbox + .base-checkbox {
    margin-top: 20px;
  }
}

.clusters-add__option {
  .button {
    margin-top: 30px
  }
}
</style>
