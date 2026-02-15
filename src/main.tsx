import React, { useState, useEffect } from 'react'
import ReactDOM from 'react-dom/client'
import { getCurrentWindow, getAllWindows } from '@tauri-apps/api/window'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import ClipboardApp from './ClipboardApp'
import SettingsApp from './SettingsApp'
import { SetupApp } from './setup'
import './index.css'

/**
 * Main app wrapper that handles first-run check and launches setup window if needed
 */
function ClipboardAppWithSetup() {
  const [loading, setLoading] = useState(true)
  const [waitingForSetup, setWaitingForSetup] = useState(false)

  useEffect(() => {
    let unlistenSetup: (() => void) | undefined

    const init = async () => {
      // Listen for setup completion event from the setup window
      // We set this up early to ensure we don't miss it
      unlistenSetup = await listen('setup_complete', async () => {
        console.log('Setup complete event received')
        setWaitingForSetup(false)
      })

      // Check if this is first run
      try {
        const isFirst = await invoke<boolean>('is_first_run')
        if (isFirst) {
          setWaitingForSetup(true)

          // In Tauri v2, if the window is in tauri.conf.json, it's already created.
          // We just need to find it and show it.
          const windows = await getAllWindows()
          const setupWin = windows.find((w) => w.label === 'setup')

          if (setupWin) {
            await setupWin.show()
            await setupWin.setFocus()
          } else {
            console.error('Setup window not found in config')
            // Attempt to create it if it somehow doesn't exist (fallback)
            const newSetupWin = new WebviewWindow('setup')
            newSetupWin.once('tauri://created', () => {
              newSetupWin.show()
              newSetupWin.setFocus()
            })
          }
        }
        setLoading(false)
      } catch (err: unknown) {
        console.error('Failed to check first run:', err)
        setLoading(false)
      }
    }

    init()

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

  if (windowLabel === 'setup') {
    return <SetupApp />
  }

  // Default to ClipboardAppWithSetup for 'main'
  return <ClipboardAppWithSetup />
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>
)
