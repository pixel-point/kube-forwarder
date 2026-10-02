'use strict'

import { join } from 'path'
import os from 'os'
import { app, BrowserWindow, Menu } from 'electron'

import configureSentry from '../common/configure-sentry'
import buildMenuTemplate from './menuTemplate'
import { checkForUpdates } from './appUpdater'
import store from './store'
import { registerIpc } from './ipc'

configureSentry()

const isDev = !app.isPackaged

let mainWindow = null

function createWindow() {
  mainWindow = new BrowserWindow({
    height: os.platform() === 'win32' ? 562 : 542,
    useContentSize: true,
    width: 800,
    titleBarStyle: 'hiddenInset',
    resizable: isDev,
    show: false,
    webPreferences: {
      preload: join(__dirname, '../preload/index.js'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false // preload needs Node to talk to ipcRenderer/@kubernetes
    }
  })

  mainWindow.on('ready-to-show', () => mainWindow.show())
  mainWindow.on('closed', () => { mainWindow = null })

  // electron-vite: dev serves the renderer over HTTP; prod loads the built file.
  if (isDev && process.env['ELECTRON_RENDERER_URL']) {
    mainWindow.loadURL(process.env['ELECTRON_RENDERER_URL'])
  } else {
    mainWindow.loadFile(join(__dirname, '../renderer/index.html'))
  }

  const menu = Menu.buildFromTemplate(buildMenuTemplate(app))
  Menu.setApplicationMenu(menu)
}

const gotLock = app.requestSingleInstanceLock()
if (!gotLock) {
  app.quit()
} else {
  app.on('second-instance', () => {
    if (mainWindow) {
      if (mainWindow.isMinimized()) mainWindow.restore()
      mainWindow.focus()
    }
  })

  app.whenReady().then(() => {
    registerIpc(() => mainWindow)
    createWindow()

    if (store.notFirstLaunch && !isDev) checkForUpdates()
    store.notFirstLaunch = true

    app.on('activate', () => {
      if (BrowserWindow.getAllWindows().length === 0) createWindow()
    })
  })

  app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') app.quit()
  })
}
