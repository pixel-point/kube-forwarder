<template>
  <table class="table forwards-table">
    <tbody>
      <tr>
        <td class="forwards-table__column-header forwards-table__column-header_name_local-port">
          Local Port
          <IconArrowDropdown thin to="right" class="forwards-table__arrow-column-divider" />
        </td>
        <td class="forwards-table__column-header forwards-table__column-header_name_remote-port">Pod Port</td>
        <td class="forwards-table__column-header forwards-table__column-header_name_actions" />
      </tr>
      <tr
        v-for="(forwardAttribute, index) in attribute.$each.$iter"
        :key="forwardAttribute.$model.id"
        class="forwards-table__column"
      >
        <td class="forwards-table__column forwards-table__column_name_local-port">
          <BaseInput
            v-model.number="forwardAttribute.localPort.$model"
            inline
            type="number"
            size="s"
            :invalid="forwardAttribute.localPort.$error"
          />
        </td>
        <td class="forwards-table__column forwards-table__column_name_remote-port">
          <BaseInput
            v-model.number="forwardAttribute.remotePort.$model"
            inline
            type="number"
            size="s"
            :invalid="forwardAttribute.remotePort.$error"
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
import uuidv1 from 'uuid'

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
  model: {
    event: 'change'
  },
  props: {
    value: { type: Array, default: () => [] }, // [{ localPort: 123, remotePort: 345, id: <uuid> }]
    attribute: { type: Object, default: null }
  },
  data() {
    return {
      newForward: this.getEmptyForward()
    }
  },
  methods: {
    getEmptyForward() {
      return { localPort: null, remotePort: null, id: uuidv1() }
    },
    create() {
      if (this.newForward.remotePort || this.newForward.localPort) {
        this.$emit('change', [...this.value, this.newForward])
        this.newForward = this.getEmptyForward()
      }
    },
    removeForward(index) {
      const nextValue = this.value.slice(0) // Clone
      nextValue.splice(index, 1) // Remove at index

      this.$emit('change', nextValue)
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
