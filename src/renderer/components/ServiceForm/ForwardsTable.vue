<template>
  <table class="table forwards-table">
    <thead>
      <tr>
        <th class="forwards-table__column-header forwards-table__column-header_name_local-port">Local port</th>
        <th class="forwards-table__column-header forwards-table__column-header_name_arrow">
          <IconArrowDropdown thin to="right"/>
        </th>
        <th class="forwards-table__column-header forwards-table__column-header_name_remote-port">Destination port</th>
        <th />
      </tr>
    </thead>
    <tbody>
      <tr
        v-for="(forward, index) in forwards"
        :key="forward.id"
        :class="{'forwards-table__column': true, 'forwards-table__column_last': index === forwards.length - 1}"
      >
        <td class="forwards-table__column forwards-table__column_name_local-port">
          <BaseInput
            v-model.number="forward.localPort"
            inline
            type="number"
            min="0"
            max="65535"
            size="s"
            :invalid="!isFieldValid(index, 'localPort')"
            @blur="save"
          />
          <ValidationErrors v-if="attribute && attribute.$each[index]" :attribute="attribute.$each[index].localPort" />
        </td>
        <td class="forwards-table__column forwards-table__column_name_arrow">
          <div class="forwards-table__divider" />
        </td>
        <td class="forwards-table__column forwards-table__column_name_remote-port">
          <BaseInput
            v-model.number="forward.remotePort"
            inline
            type="number"
            min="0"
            max="65535"
            size="s"
            :invalid="!isFieldValid(index, 'remotePort')"
            @blur.native="save"
          />
          <ValidationErrors v-if="attribute && attribute.$each[index]" :attribute="attribute.$each[index].remotePort" />
        </td>
        <td class="forwards-table__column forwards-table__column_name_actions">
          <Button
            v-if="index !== forwards.length - 1"
            theme="danger"
            size="s"
            outline
            @click="() => removeForward(index)"
          >
            ✖
          </Button>
        </td>
      </tr>
    </tbody>
  </table>
</template>

<script>
import uuidv1 from 'uuid'

import BaseInput from '../shared/form/BaseInput'
import IconArrowDropdown from '../shared/icons/IconArrowDropdown'
import Button from '../shared/Button'
import ValidationErrors from '../shared/form/ValidationErrors'

export default {
  components: {
    BaseInput,
    Button,
    IconArrowDropdown,
    ValidationErrors
  },
  model: {
    event: 'change'
  },
  props: {
    value: { type: Array, default: () => [] }, // [{ localPort: 123, remotePort: 345, id: <uuid> }]
    attribute: { type: Object, default: null }
  },
  computed: {
    forwards() {
      return [...this.value, this.getEmptyForward()]
    }
  },
  methods: {
    getEmptyForward() {
      return { localPort: null, remotePort: null, id: uuidv1() }
    },
    save() {
      const lastItem = this.forwards[this.forwards.length - 1]

      if (lastItem.remotePort || lastItem.localPort) {
        this.$emit('change', this.forwards)
      }
    },
    removeForward(index) {
      const nextValue = this.value.slice(0) // Clone
      nextValue.splice(index, 1) // Remove at index

      this.$emit('change', nextValue)
    },
    isFieldValid(index, name) {
      const attr = this.attribute && this.attribute.$each[index]
      return attr ? !attr[name].$error : true
    }
  }
}
</script>

<style lang="scss">
@import "../../assets/styles/variables";

.forwards-table {
  .forwards-table__column {
    text-align: center;
  }

  .forwards-table__column-header_name_local-port,
  .forwards-table__column_name_local-port {
    border-right: none;
  }

  .forwards-table__column-header_name_arrow,
  .forwards-table__column_name_arrow {
    border-right: none;
    border-left: none;
    color: $color-text-tertiary;
  }

  .forwards-table__column-header_name_remote-port,
  .forwards-table__column_name_remote-port {
    border-left: none;
  }

  .forwards-table__column_name_actions {
    width: 37px;

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

.forwards-table__column {
  /*padding: 2px;*/

  .validation-errors {
    margin-top: 2px;
  }
}

.forwards-table__column-header_name_arrow {
  .icon_arrow-dropdown {
    display: block;
    margin: auto;
  }
}

.forwards-table__column_name_arrow {
  padding: 0;
  position: relative;
}
</style>
