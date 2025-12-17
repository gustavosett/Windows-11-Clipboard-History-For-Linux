import React, { useState } from 'react'
import ReactDOM from 'react-dom/client'
import { getCurrentWindow } from '@tauri-apps/api/window'
import ClipboardApp from './ClipboardApp'
import SettingsApp from './SettingsApp'
import './index.css'

/**
 * Root component that routes to either ClipboardApp or SettingsApp
 * based on the current window's label
 */
export default function Root() {
  const [windowLabel] = useState<string>(() => getCurrentWindow().label)

  // Route to appropriate app based on window label
  if (windowLabel === 'settings') {
    return <SettingsApp />
  }

  // Default to ClipboardApp for 'main' and any other window
  return <ClipboardApp />
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>
)
