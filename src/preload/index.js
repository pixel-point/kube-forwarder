// The ONLY bridge between the sandboxed renderer and the main process.
// contextIsolation is on, nodeIntegration is off — the renderer sees `window.api`
// and nothing else from Node/Electron.
import { contextBridge, ipcRenderer } from 'electron'

const api = {
  app: {
    defaultKubeConfigPath: () => ipcRenderer.invoke('app:defaultKubeConfigPath')
  },
  readFile: filePath => ipcRenderer.invoke('file:read', filePath),
  writeFile: (filePath, content) => ipcRenderer.invoke('file:write', filePath, content),
  cluster: {
    check: (config, context = null) => ipcRenderer.invoke('cluster:check', config, context),
    contextsFromString: content => ipcRenderer.invoke('cluster:contextsFromString', content),
    contextsFromFile: filePath => ipcRenderer.invoke('cluster:contextsFromFile', filePath)
  },
  resources: {
    namespaces: config => ipcRenderer.invoke('resources:namespaces', config),
    list: (config, kind, namespace) => ipcRenderer.invoke('resources:list', config, kind, namespace)
  },
  connections: {
    create: payload => ipcRenderer.invoke('connections:create', payload),
    delete: service => ipcRenderer.invoke('connections:delete', service),
    onEvent: callback => {
      const listener = (_e, data) => callback(data)
      ipcRenderer.on('connection:event', listener)
      return () => ipcRenderer.removeListener('connection:event', listener)
    }
  },
  dialog: {
    messageBox: options => ipcRenderer.invoke('dialog:messageBox', options),
    errorBox: (title, message) => ipcRenderer.invoke('dialog:errorBox', title, message),
    openFile: options => ipcRenderer.invoke('dialog:openFile', options),
    saveFile: options => ipcRenderer.invoke('dialog:saveFile', options)
  },
  shell: {
    openExternal: url => ipcRenderer.invoke('shell:openExternal', url)
  },
  store: {
    getSync: () => ipcRenderer.sendSync('store:getSync'),
    set: state => ipcRenderer.invoke('store:set', state)
  }
}

contextBridge.exposeInMainWorld('api', api)
