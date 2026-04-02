const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('electronAPI', {
  // Returns the current HTML filename (e.g. "portfolio.html")
  // so each page can highlight its own primary nav link
  currentPage: () => {
    const parts = window.location.pathname.split('/');
    return parts[parts.length - 1] || 'dashboard.html';
  },

  // Navigate to a named page (relative filename)
  navigate: (page) => {
    window.location.href = page;
  },
});
