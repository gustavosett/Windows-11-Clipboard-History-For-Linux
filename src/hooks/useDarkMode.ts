import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { ThemeInfo } from '../types/clipboard'

/**
 * Query the backend for system color scheme via XDG Desktop Portal.
 * This works with COSMIC, GNOME, KDE, and other portal-compliant DEs.
 */
async function getSystemThemeFromPortal(): Promise<boolean | null> {
  try {
    const themeInfo = await invoke<ThemeInfo>('get_system_theme')
    // Only trust the result if we got it from a real source
    if (themeInfo.source !== 'default') {
      return themeInfo.prefers_dark
    }
    return null
  } catch (error) {
    console.warn('[useDarkMode] Failed to get system theme from portal:', error)
    return null
  }
}

/**
 * Check if the D-Bus event listener is active
 */
async function isEventListenerActive(): Promise<boolean> {
  try {
    return await invoke<boolean>('is_theme_listener_active')
  } catch {
    return false
  }
}

/**
 * Hook for detecting system dark mode preference.
 * Uses CSS media query with XDG Desktop Portal fallback for COSMIC DE and others.
 * Listens for D-Bus theme change events, with polling fallback (10s) if events unavailable.
 */
export function useDarkMode(): boolean {
  const [isDark, setIsDark] = useState(() => {
    if (globalThis.matchMedia) {
      return globalThis.matchMedia('(prefers-color-scheme: dark)').matches
    }
    return true // Default to dark mode
  })

  // Sync DOM with state
  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }, [isDark])

  // Track if we've checked the portal for initial theme
  const hasCheckedPortal = useRef(false)

  // Check XDG portal for initial theme (handles COSMIC and other DEs)
  useEffect(() => {
    if (hasCheckedPortal.current) return
    hasCheckedPortal.current = true

    const checkPortalTheme = async () => {
      const portalPrefersDark = await getSystemThemeFromPortal()
      if (portalPrefersDark !== null) {
        // Portal returned a valid preference, use it
        setIsDark(portalPrefersDark)
      }
    }

    checkPortalTheme()
  }, [])

  // Listen for system theme changes via media query
  useEffect(() => {
    const mediaQuery = globalThis.matchMedia('(prefers-color-scheme: dark)')

    // Listen for changes
    const handleChange = (e: MediaQueryListEvent) => {
      setIsDark(e.matches)
    }

    mediaQuery.addEventListener('change', handleChange)

    return () => {
      mediaQuery.removeEventListener('change', handleChange)
    }
  }, [])

  // Listen for theme change events from the backend (D-Bus signals)
  useEffect(() => {
    const unlistenPromise = listen<ThemeInfo>('system-theme-changed', (event) => {
      const themeInfo = event.payload
      setIsDark(themeInfo.prefers_dark)
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
      const hasEventListener = await isEventListenerActive()

      if (!hasEventListener) {
        // Event listener not available, use polling fallback
        checkInterval = setInterval(async () => {
          const portalPrefersDark = await getSystemThemeFromPortal()
          if (portalPrefersDark !== null) {
            setIsDark((prev) => {
              if (prev !== portalPrefersDark) return portalPrefersDark
              return prev
            })
          }
        }, 10000) // Check every 10 seconds
      }
    }

    setupPolling()

    return () => {
      if (checkInterval) clearInterval(checkInterval)
    }
  }, [])

  return isDark
}
