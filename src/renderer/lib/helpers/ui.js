const { dialog, app } = require('electron').remote

// @param options.details
export function showMessageBox(message, options = {}) {
  const { details } = options
  const detailsLabel = 'Details'
  const buttons = ['OK']
  if (details) buttons.push(detailsLabel)

  return new Promise((resolve) => {
    dialog.showMessageBox({ title: 'Message', message, buttons, ...options }, (...args) => resolve(args))
  }).then(args => {
    const index = args ? args[0] : null
    if (details && index === buttons.indexOf(detailsLabel)) {
      return showMessageBox(details)
    }

    return args
  })
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

export function showSaveDialog(options = {}) {
  const defaultPath = options.defaultName ? `${app.getPath('documents')}/${options.defaultName}` : undefined

  return new Promise(resolve => {
    dialog.showSaveDialog({ defaultPath, ...options }, resolve)
  })
}

export function showOpenDialog(options = {}) {
  const defaultPath = options.defaultPath || app.getPath('documents')

  return new Promise(resolve => {
    dialog.showOpenDialog({ defaultPath, ...options }, resolve)
  })
}
