import { useEffect, useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { ThemeInfo } from '../types/clipboard'

/**
 * Query the backend for system color scheme via XDG Desktop Portal.
 * This works with COSMIC, GNOME, KDE, and other portal-compliant DEs.
 */
export async function getSystemThemeFromPortal(): Promise<boolean | null> {
  try {
    const themeInfo = await invoke<ThemeInfo>('get_system_theme')
    if (themeInfo.source !== 'default') {
      return themeInfo.prefers_dark
    }
    return null
  } catch (error) {
    console.warn('[systemTheme] Failed to get system theme from portal:', error)
    return null
  }
}

/**
 * Hook for detecting system dark mode preference.
 * Uses CSS media query with XDG Desktop Portal fallback for COSMIC DE and others.
 * Listens for D-Bus theme change events, with polling fallback (30s) if events unavailable.
 */
export function useSystemThemePreference(): boolean {
  const [systemPrefersDark, setSystemPrefersDark] = useState(() => {
    if (globalThis.matchMedia) {
      return globalThis.matchMedia('(prefers-color-scheme: dark)').matches
    }
    return true
  })
  const hasCheckedPortal = useRef(false)

  // Check XDG portal for initial theme (handles COSMIC and other DEs)
  useEffect(() => {
    if (hasCheckedPortal.current) return
    hasCheckedPortal.current = true

    getSystemThemeFromPortal().then((portalPrefersDark) => {
      if (portalPrefersDark !== null) {
        setSystemPrefersDark(portalPrefersDark)
      }
    })
  }, [])

  // Listen for media query changes
  useEffect(() => {
    const mediaQuery = globalThis.matchMedia('(prefers-color-scheme: dark)')
    const handleChange = (e: MediaQueryListEvent) => {
      setSystemPrefersDark(e.matches)
    }
    mediaQuery.addEventListener('change', handleChange)
    return () => mediaQuery.removeEventListener('change', handleChange)
  }, [])

  // Listen for theme change events from the backend (D-Bus signals)
  useEffect(() => {
    const unlistenPromise = listen<ThemeInfo>('system-theme-changed', (event) => {
      const themeInfo = event.payload
      setSystemPrefersDark(themeInfo.prefers_dark)
    })

    return () => {
      unlistenPromise.then((unlisten) => unlisten())
    }
  }, [])

  // Polling fallback: Only poll if D-Bus event listener is not active
  // This handles DEs that don't support portal signals or if the listener failed
  useEffect(() => {
    let checkInterval: number | null = null

    const setupPolling = async () => {
      const hasEventListener = await invoke<boolean>('is_theme_listener_active')

      if (!hasEventListener) {
        // Event listener not available, use polling fallback
        checkInterval = setInterval(async () => {
          const portalPrefersDark = await getSystemThemeFromPortal()
          if (portalPrefersDark !== null) {
            setSystemPrefersDark(portalPrefersDark)
          }
        }, 10000) // Check every 10 seconds
      }
    }

    setupPolling()

    return () => {
      if (checkInterval) clearInterval(checkInterval)
    }
  }, [])

  return systemPrefersDark
}
