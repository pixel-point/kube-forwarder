export function SET(state, item) {
  state.items[item.id] = item
}

export function DELETE(state, id) {
  delete state.items[id]
}
