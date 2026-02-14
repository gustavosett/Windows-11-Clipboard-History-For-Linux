import React, { useState, useEffect } from 'react'
import ReactDOM from 'react-dom/client'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import ClipboardApp from './ClipboardApp'
import SettingsApp from './SettingsApp'
import './index.css'

/**
 * Main app wrapper that handles first-run check and launches setup window if needed
 */
function ClipboardAppWithSetup() {
  const [loading, setLoading] = useState(true)
  const [waitingForSetup, setWaitingForSetup] = useState(false)

  useEffect(() => {
    // Listen for setup completion event from the setup window
    // We set this up early to ensure we don't miss it
    let unlistenSetup: (() => void) | undefined

    const setupListener = async () => {
      unlistenSetup = await listen('setup_complete', async () => {
        console.log('Setup complete event received')
        setWaitingForSetup(false)
        // Show main window when setup is done
        const win = getCurrentWindow()
        await win.show()
        await win.setFocus()
      })
    }
    setupListener()

    // Check if this is first run
    invoke<boolean>('is_first_run')
      .then(async (isFirst: boolean) => {
        if (isFirst) {
          setWaitingForSetup(true)
          // Launch the setup window
          const setupWin = new WebviewWindow('setup')

          setupWin.once('tauri://created', () => {
            setupWin.show()
            setupWin.setFocus()
          })
          setupWin.once('tauri://error', (e: any) => {
            console.error('Setup window error:', e)
            // If it already exists, just show and focus
            if (typeof e === 'string' && e.includes('already exists')) {
              setupWin.show()
              setupWin.setFocus()
            }
          })

          // Fallback show
          setupWin.show()
          setupWin.setFocus()
        }
        setLoading(false)
      })
      .catch((err: any) => {
        console.error('Failed to check first run:', err)
        setLoading(false)
      })

    return () => {
      if (unlistenSetup) unlistenSetup()
    }
  }, [])

  if (loading || waitingForSetup) {
    // Show nothing while checking status or waiting for setup to complete
    // This prevents the clipboard app from trying to initialize before permissions are granted
    return null
  }

  return <ClipboardApp />
}

/**
 * Root component that routes based on the current window's label
 */
export default function Root() {
  const [windowLabel] = useState<string>(() => getCurrentWindow().label)

  // Route to appropriate app based on window label
  if (windowLabel === 'settings') {
    return <SettingsApp />
  }

  // Note: 'setup' window has its own entry point (setup.html -> src/setup.tsx)
  // so we don't need to handle it here.

  // Default to ClipboardAppWithSetup for 'main'
  return <ClipboardAppWithSetup />
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>
)
