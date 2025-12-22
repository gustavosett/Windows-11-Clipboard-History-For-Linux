import { enable, disable, isEnabled } from '@tauri-apps/plugin-autostart'
import { useState, useEffect } from 'react'

export function useAutostart() {
  const [enabled, setEnabled] = useState(false)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    checkStatus()
  }, [])

  const checkStatus = async () => {
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
  }

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
