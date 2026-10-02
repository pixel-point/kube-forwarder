// The single IPC boundary. Everything the renderer used to do directly with
// Node/Electron/k8s now goes through these handlers.
import { ipcMain, dialog, shell, app } from 'electron'
import { promises as fs } from 'fs'
import * as path from 'path'

import * as k8s from './k8s'
import * as persist from './persist'

let getWindow = () => null

export function registerIpc(windowGetter) {
  getWindow = windowGetter

  // Push port-forward state changes into the renderer's vuex Connections module.
  k8s.setEmitter((type, payload) => {
    const win = getWindow()
    if (win && !win.isDestroyed()) win.webContents.send('connection:event', { type, payload })
  })

  // --- app paths -----------------------------------------------------------
  ipcMain.handle('app:defaultKubeConfigPath', () => path.join(app.getPath('home'), '.kube/config'))

  // --- file read (kubeconfig import) --------------------------------------
  ipcMain.handle('file:read', async (_e, filePath) => {
    try {
      const stat = await fs.stat(filePath)
      const content = await fs.readFile(filePath, { encoding: 'utf8' })
      return { size: stat.size, content }
    } catch (error) {
      return { error: error.message }
    }
  })

  ipcMain.handle('file:write', async (_e, filePath, content) => {
    try {
      await fs.writeFile(filePath, content)
      return {}
    } catch (error) {
      return { error: error.message }
    }
  })

  // --- cluster / kubeconfig ------------------------------------------------
  ipcMain.handle('cluster:check', (_e, config, context) => k8s.checkConnection(config, context))
  ipcMain.handle('cluster:contextsFromString', (_e, content) => k8s.contextsFromString(content))
  ipcMain.handle('cluster:contextsFromFile', (_e, filePath) => k8s.contextsFromFile(filePath))

  // --- resource listing ----------------------------------------------------
  ipcMain.handle('resources:namespaces', (_e, config) => k8s.listNamespaces(config))
  ipcMain.handle('resources:list', (_e, config, kind, namespace) => k8s.listResources(config, kind, namespace))

  // --- port-forward connections -------------------------------------------
  ipcMain.handle('connections:create', (_e, payload) => k8s.createConnection(payload))
  ipcMain.handle('connections:delete', (_e, service) => k8s.deleteConnection(service))

  // --- dialogs / shell -----------------------------------------------------
  ipcMain.handle('dialog:messageBox', (_e, options) => dialog.showMessageBox(getWindow(), options))
  ipcMain.handle('dialog:errorBox', (_e, title, message) => dialog.showErrorBox(title, message))

  ipcMain.handle('dialog:openFile', async (_e, options = {}) => {
    const defaultPath = options.defaultPath || app.getPath('documents')
    const { canceled, filePaths } = await dialog.showOpenDialog(getWindow(), { defaultPath, ...options })
    return canceled ? undefined : filePaths
  })

  ipcMain.handle('dialog:saveFile', async (_e, options = {}) => {
    const defaultPath = options.defaultName
      ? path.join(app.getPath('documents'), options.defaultName)
      : options.defaultPath
    const { canceled, filePath } = await dialog.showSaveDialog(getWindow(), { defaultPath, ...options })
    return canceled ? undefined : filePath
  })

  ipcMain.handle('shell:openExternal', (_e, url) => shell.openExternal(url))

  // --- vuex persistence ----------------------------------------------------
  ipcMain.on('store:getSync', e => { e.returnValue = persist.getState() })
  ipcMain.handle('store:set', (_e, state) => persist.setState(state))
}
