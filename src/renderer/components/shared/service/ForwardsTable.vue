<template>
  <table class="table forwards-table">
    <tbody>
      <tr>
        <td class="forwards-table__column-header forwards-table__column-header_name_local-port">
          Local Port
          <IconArrowDropdown thin to="right" class="forwards-table__arrow-column-divider" />
        </td>
        <td class="forwards-table__column-header forwards-table__column-header_name_remote-port">Resource Port</td>
        <td class="forwards-table__column-header forwards-table__column-header_name_actions" />
      </tr>
      <tr
        v-for="(forward, index) in modelValue"
        :key="forward.id"
        class="forwards-table__column"
      >
        <td class="forwards-table__column forwards-table__column_name_local-port">
          <BaseInput
            :model-value="forward.localPort"
            inline
            type="number"
            size="s"
            :invalid="rowError(index, 'localPort')"
            @update:model-value="value => updatePort(index, 'localPort', value)"
          />
        </td>
        <td class="forwards-table__column forwards-table__column_name_remote-port">
          <BaseInput
            :model-value="forward.remotePort"
            inline
            type="number"
            size="s"
            :invalid="rowError(index, 'remotePort')"
            @update:model-value="value => updatePort(index, 'remotePort', value)"
          />
        </td>
        <td class="forwards-table__column forwards-table__column_name_actions">
          <Button theme="danger" size="s" layout="text" @click="() => removeForward(index)">
            <IconCross />
          </Button>
        </td>
      </tr>

      <tr :key="newForward.id">
        <td class="forwards-table__column forwards-table__column_name_local-port">
          <BaseInput
            v-model.number="newForward.localPort"
            inline
            type="number"
            size="s"
            @blur="create"
          />
        </td>
        <td class="forwards-table__column forwards-table__column_name_remote-port">
          <BaseInput
            v-model.number="newForward.remotePort"
            inline
            type="number"
            size="s"
            @blur="create"
          />
        </td>
        <td class="forwards-table__column forwards-table__column_name_actions" />
      </tr>
    </tbody>
  </table>
</template>

<script>
import { v1 as uuidv1 } from 'uuid'

import BaseInput from '../form/BaseInput'
import IconArrowDropdown from '../icons/IconArrowDropdown'
import IconCross from '../icons/IconCross'
import Button from '../Button'

export default {
  components: {
    BaseInput,
    Button,
    IconArrowDropdown,
    IconCross
  },
  props: {
    modelValue: { type: Array, default: () => [] }, // [{ localPort: 123, remotePort: 345, id: <uuid> }]
    attribute: { type: Object, default: null }
  },
  emits: ['update:modelValue'],
  data() {
    return {
      newForward: this.getEmptyForward()
    }
  },
  watch: {
    'newForward.localPort': function(next, prev) {
      if (next && prev !== next) {
        this.newForward.remotePort = next
      }
    },
    'newForward.remotePort': function(next, prev) {
      if (next && prev !== next) {
        this.newForward.localPort = next
      }
    }
  },
  methods: {
    getEmptyForward() {
      return { localPort: null, remotePort: null, id: uuidv1() }
    },
    rowError(index, key) {
      const row = this.attribute && this.attribute.rows[index]
      return row ? row[key].$error : false
    },
    updatePort(index, key, value) {
      const number = parseFloat(value)
      const next = this.modelValue.map((forward, i) =>
        i === index ? { ...forward, [key]: Number.isNaN(number) ? value : number } : forward
      )
      this.$emit('update:modelValue', next)
    },
    create() {
      if (this.newForward.remotePort || this.newForward.localPort) {
        this.$emit('update:modelValue', [...this.modelValue, this.newForward])
        this.newForward = this.getEmptyForward()
      }
    },
    removeForward(index) {
      const nextValue = this.modelValue.slice(0) // Clone
      nextValue.splice(index, 1) // Remove at index

      this.$emit('update:modelValue', nextValue)
    }
  }
}
</script>

<style lang="scss">
@import "../../../assets/styles/variables";

.forwards-table {
  .forwards-table__column {
    text-align: center;
  }

  .forwards-table__column-header_name_local-port {
    border-right: none;
    width: 341px;
    position: relative;
  }

  .forwards-table__arrow-column-divider {
    color: $color-text-quaternary;
    position: absolute;
    right: -5px;
    top: 10px;
  }

  .forwards-table__column-header_name_remote-port {
    width: 341px;
    border-left: none;
  }

  .forwards-table__column_name_actions {
    .button {
      width: 30px;
      padding: 0;
    }
  }

  .forwards-table__column_name_local-port,
  .forwards-table__column_name_remote-port {
    .base-input {
      text-align: center;
      width: 100%;
    }
  }
}

.forwards-table__divider {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 1px;
  left: 50%;
  background-color: $table-border-color;
}
</style>
