import React from 'react'
import ReactDOM from 'react-dom/client'
import { SetupWizard } from './components/SetupWizard'
import './index.css'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'

export function SetupApp() {
  const handleComplete = async () => {
    try {
      console.log('SetupApp: invoking finish_setup command')
      await invoke('finish_setup')
    } catch (err) {
      console.error('Failed to finish setup:', err)
      // Fallback: try to close window manually
      await getCurrentWindow().close()
    }
  }

  return (
    <div className="h-screen w-screen overflow-hidden bg-transparent">
      {/* Pass a dummy onComplete that just closes the window, 
            or modify SetupWizard to handle it internally if needed.
            For now, we wrap it to ensure window closure. */}
      <SetupWizard onComplete={handleComplete} />
    </div>
  )
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <SetupApp />
  </React.StrictMode>
)
