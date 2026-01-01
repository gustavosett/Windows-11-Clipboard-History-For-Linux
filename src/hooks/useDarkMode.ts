import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
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
 * Hook for detecting system dark mode preference.
 * Uses CSS media query with XDG Desktop Portal fallback for COSMIC DE and others.
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

  // Periodically check portal for theme changes (handles DEs that don't propagate to media query)
  useEffect(() => {
    const checkInterval = setInterval(async () => {
      const portalPrefersDark = await getSystemThemeFromPortal()
      if (portalPrefersDark !== null) {
        setIsDark((prev) => {
          if (prev !== portalPrefersDark) return portalPrefersDark
          return prev
        })
      }
    }, 5000) // Check every 5 seconds

    return () => clearInterval(checkInterval)
  }, [])

  return isDark
}
