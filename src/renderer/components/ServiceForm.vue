<template>
  <div class="service-form">
    <Header :back-path="backPath" />

    <BaseForm @submit="handleSubmit" multicolumn>
      <fieldset>
        <ControlGroup label="Cluster Name" size="2" :attribute="$v.attributes.clusterId">
          <BaseSelect v-model="$v.attributes.clusterId.$model" :options="clusterOptions" />
        </ControlGroup>

        <ControlGroup label="Namespace" size="2" :attribute="$v.attributes.namespace">
          <BaseInput v-model="$v.attributes.namespace.$model" />
        </ControlGroup>

        <ControlGroup label="Workload Type" size="2" :attribute="$v.attributes.workloadType">
          <BaseSelect
            v-model="$v.attributes.workloadType.$model"
            :options="workloadTypeOptions"
            placeholder="Select Workload Type"
          />
        </ControlGroup>

        <ControlGroup
          label="Workload Name"
          size="2"
          :attribute="$v.attributes.workloadName"
          :disabled="!attributes.workloadType"
        >
          <template v-slot="slotProps">
            <BaseInput v-model="slotProps.attribute.$model" v-bind="slotProps" />
          </template>
        </ControlGroup>

        <ControlGroup label="Alias" size="2" :attribute="$v.attributes.alias">
          <BaseInput v-model="$v.attributes.alias.$model" placeholder="Optional..." />
        </ControlGroup>

        <ControlGroup label="Ports Forwarding">
          <ForwardsTable v-model="$v.attributes.forwards.$model" :attribute="$v.attributes.forwards" />
        </ControlGroup>
      </fieldset>

      <div class="control-actions">
        <Button theme="danger" layout="outline" :to="backPath">Cancel</Button>
        <div class="space" />
        <div class="control-actions__error">{{ error }}</div>
        <Button type="submit" theme="primary" :disabled="$v.$invalid">{{ submitButtonTitle }}</Button>
      </div>
    </BaseForm>
  </div>
</template>

<script>
import cloneDeep from 'clone-deep'
import { mapActions } from 'vuex'
import { required, minLength, integer, between } from 'vuelidate/lib/validators'
import { validationMixin } from 'vuelidate'

import * as workloadTypes from '../lib/constants/workload-types'

import BaseForm from './shared/form/BaseForm'
import BaseInput from './shared/form/BaseInput'
import BaseSelect from './shared/form/BaseSelect'
import Button from './shared/Button'
import Header from './shared/Header'
import ForwardsTable from './ServiceForm/ForwardsTable'
import ControlGroup from './shared/form/ControlGroup'

export default {
  components: {
    ControlGroup,
    BaseInput,
    BaseForm,
    Button,
    Header,
    BaseSelect,
    ForwardsTable
  },
  mixins: [validationMixin],
  props: {
    editId: { type: String, default: null },
    initialAttributes: { type: Object, default: () => ({}) }
  },
  validations: {
    attributes: {
      clusterId: { required },
      alias: {},
      namespace: { required },
      workloadType: {
        required,
        oneOf: (value) => Object.values(workloadTypes).includes(value)
      },
      workloadName: { required },
      forwards: {
        required,
        minLength: minLength(1),
        $each: {
          localPort: { required, integer, between: between(0, 65535) },
          remotePort: { required, integer, between: between(0, 65535) }
        }
      }
    }
  },
  data() {
    return {
      error: null,
      attributes: {
        ...this.getEmptyAttributes(),
        clusterId: this.$route.params.clusterId,
        ...cloneDeep(this.initialAttributes)
      }
    }
  },
  computed: {
    clusterOptions() {
      return Object.values(this.$store.state.Clusters.items).map(x => [x.id, x.name])
    },
    workloadTypeOptions() {
      return [
        [workloadTypes.POD, 'Pod'],
        [workloadTypes.DEPLOYMENT, 'Deployment']
      ]
    },
    submitButtonTitle() {
      return this.editId ? `Save` : 'Add a service'
    },
    backPath() {
      return '/'
    }
  },
  methods: {
    ...mapActions('Services', ['createService']),
    getEmptyAttributes() {
      return {
        clusterId: null,
        name: '',
        namespace: '',
        workloadType: null,
        workloadName: '',
        forwards: []
      }
    },
    handleSubmit() {
      const action = this.editId ? 'Services/updateService' : 'Services/createService'

      this.$store.dispatch(action, { ...this.attributes }).then(result => {
        if (result.success) {
          this.$router.push('/')
        } else {
          this.error = JSON.stringify(result.errors)
        }
      })
    }
  }
}
</script>

<style scoped lang="scss">
.service-form {
  padding: 20px;

  .header {
    margin-bottom: 20px
  }

  .base-form {
    display: flex;
    flex-direction: column;

    /*fieldset {*/
    /*  overflow: auto;*/
    /*}*/
    /*max-height: 409px;*/
  }
}
</style>
