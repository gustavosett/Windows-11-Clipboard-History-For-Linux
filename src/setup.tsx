import { SetupWizard } from './components/SetupWizard'
import './index.css'
import { invoke } from '@tauri-apps/api/core'
import { useTranslation } from 'react-i18next'

export function SetupApp() {
  const { t } = useTranslation()
  const handleComplete = async () => {
    try {
      console.log('SetupApp: invoking finish_setup command')
      await invoke('finish_setup')
    } catch (err) {
      console.error('Failed to finish setup:', err)
      // Do not close the window on failure; keep it open so the user can retry.
      // If we close it while first_run is still true, the backend will exit the app.
      if (typeof window !== 'undefined') {
        alert(t('setup.finish_failed'))
      }
    }
  }

  return (
    <div className="h-screen w-screen overflow-hidden bg-transparent">
      {/* Pass handleComplete which invokes the backend finish_setup command 
            to mark setup as done, close this window, and show the main app. */}
      <SetupWizard onComplete={() => void handleComplete()} />
    </div>
  )
}
