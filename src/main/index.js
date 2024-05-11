'use strict'
/* eslint-disable import/first */
import { app, BrowserWindow, Menu } from 'electron'
import os from 'os'
import path from 'path'
import buildMenuTemplate from './menuTemplate'
import { checkForUpdates } from './appUpdater'
import store from './store'
import * as remoteMain from '@electron/remote/main'
remoteMain.initialize()
import debug from 'electron-debug'
if (process.env.NODE_ENV === 'development') {
  debug()
}
/* eslint-enable import/first */

/**
 * Set `__static` path to static files in production
 * https://simulatedgreg.gitbooks.io/electron-vue/content/en/using-static-assets.html
 */
if (process.env.NODE_ENV !== 'development') {
  global.__static = path.join(__dirname, 'static').replace(/\\/g, '\\\\')
}

let mainWindow
const winURL = process.env.NODE_ENV === 'development'
  ? 'http://localhost:8080'
  : `file://${path.join(__dirname, 'index.html')}`

function createWindow() {
  mainWindow = new BrowserWindow({
    height: os.platform() === 'win32' ? 562 : 542,
    useContentSize: true,
    width: 800,
    titleBarStyle: 'hiddenInset',
    resizable: process.env.NODE_ENV === 'development',
    webPreferences: {
      plugins: true,
      nodeIntegration: true,
      contextIsolation: false,
      backgroundThrottling: false,
      nativeWindowOpen: false,
      webSecurity: false
    }
  })

  mainWindow.loadURL(winURL)

  mainWindow.on('closed', () => {
    mainWindow = null
  })

  remoteMain.enable(mainWindow.webContents)

  // Open DevTools only in development mode
  if (process.env.NODE_ENV === 'development') {
    mainWindow.webContents.openDevTools()
  }

  const menuTemplate = buildMenuTemplate(app)
  const menu = Menu.buildFromTemplate(menuTemplate)
  Menu.setApplicationMenu(menu)
}

app.on('ready', () => {
  createWindow()

  if (store.notFirstLaunch) {
    if (process.env.NODE_ENV === 'production') {
      checkForUpdates()
    }
  }

  store.notFirstLaunch = true
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})

app.on('activate', () => {
  if (mainWindow === null) {
    createWindow()
  }
})
