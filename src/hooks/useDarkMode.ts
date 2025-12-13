import { useState, useEffect } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'

/**
 * Hook for detecting system dark mode preference
 */
export function useDarkMode(): boolean {
  const [isDark, setIsDark] = useState(() => {
    if (typeof window !== 'undefined') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches
    }
    return true // Default to dark mode
  })

  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')

    const handleChange = (e: MediaQueryListEvent) => {
      setIsDark(e.matches)
    }

    // Modern browsers
    mediaQuery.addEventListener('change', handleChange)

    getCurrentWindow()
      .theme()
      .then((theme) => {
        if (theme) {
          setIsDark(theme === 'dark')
        }
      })
      .catch(console.error)

    return () => {
      mediaQuery.removeEventListener('change', handleChange)
    }
  }, [])

  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }, [isDark])

  return isDark
}
