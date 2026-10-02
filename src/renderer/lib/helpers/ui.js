// Dialogs run in the main process; the renderer reaches them through window.api.

// @param options.details — when set, adds a "Details" button that opens a follow-up box.
export async function showMessageBox(message, options = {}) {
  const { details } = options
  const detailsLabel = 'Details'
  const buttons = options.buttons ? options.buttons.slice(0) : ['OK']
  if (details) buttons.push(detailsLabel)

  const { response, checkboxChecked } = await window.api.dialog.messageBox({
    title: 'Message', message, buttons, ...options
  })

  if (details && response === buttons.indexOf(detailsLabel)) {
    return showMessageBox(details)
  }

  return [response, checkboxChecked]
}

export async function showConfirmBox(message, options = {}) {
  const [buttonIndex] = await showMessageBox(message, {
    title: 'Confirm',
    buttons: ['OK', 'Cancel'],
    ...options
  })

  return buttonIndex === 0
}

export function showErrorBox(message, title = 'Error') {
  window.api.dialog.errorBox(title, message)
}

export function showSaveDialog(options = {}) {
  return window.api.dialog.saveFile(options)
}

export function showOpenDialog(options = {}) {
  return window.api.dialog.openFile(options)
}
