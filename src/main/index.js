"use strict";

// require Sentry as soon as possible
import configureSentry from "../common/configure-sentry";
configureSentry();

/* eslint-disable import/first */
import { app, BrowserWindow, Menu, Tray, nativeImage } from "electron";
import os from "os";
const path = require("path");
import buildMenuTemplate from "./menuTemplate";
import { checkForUpdates } from "./appUpdater";
import store from "./store";

/* eslint-enable import/first */

/**
 * Set `__static` path to static files in production
 * https://simulatedgreg.gitbooks.io/electron-vue/content/en/using-static-assets.html
 */
let trayWindow;

let imgPath;
let image;
let tray;

if (process.env.NODE_ENV !== "development") {
  global.__static = path.join(__dirname, "/static").replace(/\\/g, "\\\\");
  imgPath = path.join(process.resourcesPath, "app-icon-tray.png");
  image = nativeImage.createFromPath(imgPath);
} else {
  imgPath = path.join(__dirname, "app-icon-tray.png");
  image = nativeImage.createFromPath(imgPath);
}

const winURL =
  process.env.NODE_ENV === "development"
    ? `http://localhost:9080`
    : `file://${__dirname}/index.html`;

    function createWindow() {
      let window = new BrowserWindow({
        height: os.platform() === "win32" ? 562 : 542,
        useContentSize: true,
        width: 800,
        titleBarStyle: "hiddenInset",
        resizable: process.env.NODE_ENV === "development",
        webPreferences: {
          webSecurity: false,
          contextIsolation: false,
          nodeIntegration: true,
          enableRemoteModule: true,
        },
      });
      window.loadURL(winURL)
      window.on("closed", () => {
        window = null;
      });
    
      const menuTemplate = buildMenuTemplate(app);
      const menu = Menu.buildFromTemplate(menuTemplate);
      Menu.setApplicationMenu(menu);
      return window;
    }
    
    app.on("ready", () => {
      tray = new Tray(image);
    
      const contextMenu = Menu.buildFromTemplate([
        {
          label: "Quit",
          click: () => {
            app.quit();
          },
        },
      ]);
    
      tray.on("click", (event) => {
        if (event.button === 2) {
          // If the right button was clicked, show the context menu
          tray.popUpContextMenu(contextMenu);
        } else {
          // If the left button was clicked, show or hide the tray window
          if (trayWindow) {
            if (trayWindow.isVisible()) {
              trayWindow.hide();
            } else {
              trayWindow.show();
            }
          } else {
            trayWindow = createWindow();
    
            // Get the bounds of the tray icon
            const trayBounds = tray.getBounds();
    
            // Calculate the window x and y position
            const windowBounds = trayWindow.getBounds();
            const x = Math.round(
              trayBounds.x + trayBounds.width / 2 - windowBounds.width / 2
            );
            const y = Math.round(trayBounds.y + trayBounds.height);
    
            // Set the window position
            trayWindow.setPosition(x, y);
    
            // Show the window
            trayWindow.show();
          }
        }
      });
    
      tray.on("right-click", () => {
        tray.popUpContextMenu(contextMenu);
      });
    
      if (store.notFirstLaunch) {
        if (process.env.NODE_ENV === "production") {
          checkForUpdates();
        }
      }
    
      store.notFirstLaunch = true;
    });
    
    app.on("window-all-closed", () => {
      if (process.platform !== "darwin") {
        app.quit();
      }
    });
    
  