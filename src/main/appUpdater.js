import { join } from 'path'
import { autoUpdater } from 'electron-updater'
import { app, dialog } from 'electron'

autoUpdater.autoDownload = false
autoUpdater.enableUserAwareness = false

if (!app.isPackaged) {
  autoUpdater.updateConfigPath = join(__dirname, 'dev-app-update.yml')
  autoUpdater.currentVersion = app.getVersion()
}

autoUpdater.on('update-available', () => {
  dialog.showMessageBox({
    type: 'info',
    title: 'Found Updates',
    message: 'Found updates, do you want update now?',
    buttons: ['Sure', 'No']
  }).then(({ response }) => {
    if (response === 0) {
      autoUpdater.downloadUpdate()
      autoUpdater.enableUserAwareness = true
    }
  })
})

autoUpdater.on('update-not-available', () => {
  if (autoUpdater.enableUserAwareness) {
    dialog.showMessageBox({
      title: 'No Updates',
      message: 'Current version is up-to-date.'
    })
  }

  autoUpdater.enableUserAwareness = false
})

autoUpdater.on('update-downloaded', () => {
  dialog.showMessageBox({
    title: 'Install Updates',
    message: 'Updates downloaded, application will be quit for update...'
  }).then(() => {
    setImmediate(() => autoUpdater.quitAndInstall())
  })
})

autoUpdater.on('error', message => {
  if (autoUpdater.enableUserAwareness) {
    dialog.showMessageBox({
      title: 'Error',
      message: `Sorry, there was a problem updating the application. Please, try again later.\n\n${message}`
    })
  }

  autoUpdater.enableUserAwareness = false
})

export function checkForUpdates() {
  autoUpdater.checkForUpdates()
}

export function manuallyCheckForUpdates() {
  autoUpdater.enableUserAwareness = true
  autoUpdater.checkForUpdates()
}
