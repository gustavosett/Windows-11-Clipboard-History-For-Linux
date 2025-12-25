import { invoke } from '@tauri-apps/api/core'
import { useState, useEffect, useCallback } from 'react'

// Custom autostart commands that use the wrapper script on Linux
// instead of the direct binary, ensuring proper environment setup
const enable = () => invoke('autostart_enable')
const disable = () => invoke('autostart_disable')
const isEnabled = () => invoke<boolean>('autostart_is_enabled')
const migrate = () => invoke<boolean>('autostart_migrate')

export function useAutostart() {
  const [enabled, setEnabled] = useState(false)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const checkStatus = useCallback(async () => {
    try {
      const value = await isEnabled()
      setEnabled(value)
      setError(null)
    } catch (e) {
      console.error('Failed to check autostart status:', e)
      setError(String(e))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    const migrateAndCheck = async () => {
      try {
        // First, try to migrate any old autostart entries
        const migrated = await migrate()
        if (migrated) {
          console.log('[Autostart] Migrated old autostart entry to use wrapper')
        }
        // Then check the current status
        await checkStatus()
      } catch (e) {
        console.error('Failed to migrate/check autostart:', e)
        setError(String(e))
        setLoading(false)
      }
    }

    // On mount, try to migrate old autostart entries and check status
    migrateAndCheck()
  }, [checkStatus])

  const toggle = async (): Promise<boolean> => {
    setLoading(true)
    setError(null)
    try {
      if (enabled) {
        await disable()
        setEnabled(false)
        return false
      } else {
        await enable()
        setEnabled(true)
        return true
      }
    } catch (e) {
      console.error('Failed to toggle autostart:', e)
      setError(String(e))
      return enabled
    } finally {
      setLoading(false)
    }
  }

  const enableAutostart = async (): Promise<boolean> => {
    if (enabled) return true
    setLoading(true)
    setError(null)
    try {
      await enable()
      setEnabled(true)
      return true
    } catch (e) {
      console.error('Failed to enable autostart:', e)
      setError(String(e))
      return false
    } finally {
      setLoading(false)
    }
  }

  const disableAutostart = async (): Promise<boolean> => {
    if (!enabled) return true
    setLoading(true)
    setError(null)
    try {
      await disable()
      setEnabled(false)
      return true
    } catch (e) {
      console.error('Failed to disable autostart:', e)
      setError(String(e))
      return false
    } finally {
      setLoading(false)
    }
  }

  return {
    enabled,
    loading,
    error,
    toggle,
    enableAutostart,
    disableAutostart,
    checkStatus,
  }
}
