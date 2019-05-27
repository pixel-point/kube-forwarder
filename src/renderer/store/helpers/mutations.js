import Vue from 'vue'

export function SET(state, item) {
  Vue.set(state.items, item.id, item)
}

export function DELETE(state, id) {
  Vue.delete(state.items, id)
}
