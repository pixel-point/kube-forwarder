'use strict'

// require Sentry as soon as possible
import configureSentry from '../common/configure-sentry'
configureSentry()

/* eslint-disable import/first */
import { app, BrowserWindow, Menu } from 'electron'
import os from "os"

import buildMenuTemplate from './menuTemplate'
import { checkForUpdates } from './appUpdater'
import store from './store'
/* eslint-enable import/first */

/**
 * Set `__static` path to static files in production
 * https://simulatedgreg.gitbooks.io/electron-vue/content/en/using-static-assets.html
 */
if (process.env.NODE_ENV !== 'development') {
  global.__static = require('path').join(__dirname, '/static').replace(/\\/g, '\\\\')
}

if (typeof app.setAboutPanelOptions === 'function' ) {
  app.setAboutPanelOptions({
    copyright: 'Copyright © 2019 Pixel Point'
  })
}

let mainWindow
const winURL = process.env.NODE_ENV === 'development'
  ? `http://localhost:9080`
  : `file://${__dirname}/index.html`

function createWindow() {
  /**
   * Initial window options
   */
  mainWindow = new BrowserWindow({
    height: os.platform() === 'win32' ? 562 : 542,
    useContentSize: true,
    width: 800,
    titleBarStyle: 'hiddenInset',
    resizable: process.env.NODE_ENV === 'development',
    webPreferences: {
      nodeIntegration: true
    }
  })

  mainWindow.loadURL(winURL)

  mainWindow.on('closed', () => {
    mainWindow = null
  })

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
