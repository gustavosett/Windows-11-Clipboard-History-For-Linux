import React, { useState, useEffect } from 'react'
import ReactDOM from 'react-dom/client'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import ClipboardApp from './ClipboardApp'
import SettingsApp from './SettingsApp'
import { SetupWizard } from './components/SetupWizard'
import './index.css'

/**
 * Main app wrapper that handles first-run setup wizard
 */
function ClipboardAppWithSetup() {
  const [showWizard, setShowWizard] = useState(false)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    // Check if this is first run
    invoke<boolean>('is_first_run')
      .then((isFirst) => {
        setShowWizard(isFirst)
        setLoading(false)
      })
      .catch((err) => {
        console.error('Failed to check first run:', err)
        setLoading(false)
      })

    // Listen for reset-to-defaults event from settings
    let isMounted = true
    let unlistenFn: (() => void) | null = null

    listen('show-setup-wizard', () => {
      if (isMounted) {
        setShowWizard(true)
      }
    }).then((fn) => {
      if (isMounted) {
        unlistenFn = fn
      } else {
        // Component already unmounted, clean up immediately
        fn()
      }
    })

    return () => {
      isMounted = false
      unlistenFn?.()
    }
  }, [])

  const handleWizardComplete = () => {
    setShowWizard(false)
  }

  if (loading) {
    // Show nothing while checking first run status
    return null
  }

  return (
    <>
      {showWizard && <SetupWizard onComplete={handleWizardComplete} />}
      <ClipboardApp />
    </>
  )
}

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

  // Default to ClipboardApp with setup wizard for 'main' and any other window
  return <ClipboardAppWithSetup />
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>
)
