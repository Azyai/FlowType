import React from 'react';
import ReactDOM from 'react-dom/client';

import App from './App';
import './styles/index.css';

const blockedDevtoolShortcuts = new Set(['F12', 'I', 'J', 'C']);

window.addEventListener(
  'keydown',
  (event) => {
    const key = event.key.toUpperCase();
    const openDevtoolsShortcut =
      key === 'F12' || ((event.ctrlKey || event.metaKey) && event.shiftKey && blockedDevtoolShortcuts.has(key));

    if (openDevtoolsShortcut) {
      event.preventDefault();
      event.stopPropagation();
    }
  },
  true
);

window.addEventListener(
  'contextmenu',
  (event) => {
    const target = event.target;
    if (!(target instanceof HTMLElement) || !target.closest('[data-allow-context-menu="true"]')) {
      event.preventDefault();
      event.stopPropagation();
    }
  },
  true
);

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
