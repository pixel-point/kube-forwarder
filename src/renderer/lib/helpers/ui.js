const { dialog, app } = require('electron').remote

// @param options.details
export async function showMessageBox(message, options = {}) {
  const { details } = options
  const detailsLabel = 'Details'
  const buttons = options.buttons ? options.buttons.slice(0) : ['OK']
  if (details) buttons.push(detailsLabel)

  const args = await dialog.showMessageBox({ title: 'Message', message, buttons, ...options });
  const {checkboxChecked, response: index} = args;
  if (details && index === buttons.indexOf(detailsLabel)) {
    return showMessageBox(details)
  }

  return [index, checkboxChecked];
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
  dialog.showErrorBox(title, message)
}

export async function showSaveDialog(options = {}) {
  const defaultPath = options.defaultName ? `${app.getPath('documents')}/${options.defaultName}` : undefined
  const {filePath} = await dialog.showSaveDialog({ defaultPath, ...options });
  return filePath;
}

export function showOpenDialog(options = {}) {
  const defaultPath = options.defaultPath || app.getPath('documents')

  return new Promise(resolve => {
    dialog.showOpenDialog({ defaultPath, ...options }, resolve)
  })
}
