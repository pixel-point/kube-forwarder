'use strict'

import { app, BrowserWindow, Menu } from 'electron'

import buildMenuTemplate from './menuTemplate'
import { checkForUpdates } from './appUpdater'
import store from './store'

/**
 * Set `__static` path to static files in production
 * https://simulatedgreg.gitbooks.io/electron-vue/content/en/using-static-assets.html
 */
if (process.env.NODE_ENV !== 'development') {
  global.__static = require('path').join(__dirname, '/static').replace(/\\/g, '\\\\')
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
    height: 563,
    useContentSize: true,
    width: 1000,
    titleBarStyle: 'hiddenInset',
    webPreferences: {
      nodeIntegration: true
    }
  })

  if (process.env.NODE_ENV !== 'production') {
    mainWindow.webContents.openDevTools({ mode: 'bottom' })
  }

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
    checkForUpdates()
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
