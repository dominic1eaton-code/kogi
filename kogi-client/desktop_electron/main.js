const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('node:path');

const PAGE_TITLES = {
  'dashboard.html':     'KOGI · Dashboard',
  'dashboard.html': 'KOGI · Dashboard',
  'portfolio.html': 'KOGI · Portfolio',
  'wallet.html':    'KOGI · Wallet',
  'office.html':    'KOGI · Office',
  'profile.html':   'KOGI · Profile',
  'settings.html':  'KOGI · Settings',
};

function createWindow() {
  const win = new BrowserWindow({
    width: 1280,
    height: 800,
    minWidth: 900,
    minHeight: 600,
    backgroundColor: '#091310',
    titleBarStyle: process.platform === 'darwin' ? 'hiddenInset' : 'default',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
    },
    title: 'KOGI',
  });

  win.loadFile('dashboard.html');

  // Update window title whenever the page changes
  win.webContents.on('did-navigate', (_event, url) => {
    const filename = path.basename(new URL(url).pathname);
    const title = PAGE_TITLES[filename] || 'KOGI';
    win.setTitle(title);
  });

  // Keep navigation confined to local app files
  win.webContents.on('will-navigate', (event, url) => {
    try {
      const parsed = new URL(url);
      // Allow only file:// protocol navigations (local pages)
      if (parsed.protocol !== 'file:') {
        event.preventDefault();
      }
    } catch {
      event.preventDefault();
    }
  });

  // Block any attempt to open a new browser window
  win.webContents.setWindowOpenHandler(() => ({ action: 'deny' }));
}

app.whenReady().then(() => {
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});
