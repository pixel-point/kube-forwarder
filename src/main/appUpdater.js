import { autoUpdater } from 'electron-updater'
import packageJson from '../../package'
import { dialog } from 'electron'

autoUpdater.autoDownload = false
autoUpdater.enableUserAwareness = false

if (process.env.NODE_ENV !== 'production') {
  const path = require('path')
  autoUpdater.updateConfigPath = path.join(__dirname, 'dev-app-update.yml')
  // noinspection JSUnresolvedVariable
  autoUpdater.currentVersion = packageJson.version
}

autoUpdater.on('update-available', () => {
  dialog.showMessageBox({
    type: 'info',
    title: 'Found Updates',
    message: 'Found updates, do you want update now?',
    buttons: ['Sure', 'No']
  }, (buttonIndex) => {
    if (buttonIndex === 0) {
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
  }, () => {
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
