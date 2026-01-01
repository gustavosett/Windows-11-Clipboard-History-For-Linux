import { useEffect } from 'react'
import { useSystemThemePreference } from '../utils/systemTheme'

/**
 * Hook for detecting system dark mode preference and syncing with DOM.
 * Uses CSS media query with XDG Desktop Portal fallback for COSMIC DE and others.
 * Listens for D-Bus theme change events, with polling fallback if events unavailable.
 */
export function useDarkMode(): boolean {
  const isDark = useSystemThemePreference()

  // Sync DOM with state
  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }, [isDark])

  return isDark
}
