import { useState, useCallback, useEffect, useRef } from 'react'
import { clsx } from 'clsx'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { useClipboardHistory } from './hooks/useClipboardHistory'
import { HistoryItem } from './components/HistoryItem'
import { TabBar, TabBarRef } from './components/TabBar'
import { Header } from './components/Header'
import { EmptyState } from './components/EmptyState'
import { DragHandle } from './components/DragHandle'
import { EmojiPicker } from './components/EmojiPicker'
import { GifPicker } from './components/GifPicker'
import type { ActiveTab } from './types/clipboard'

/** User settings type matching the Rust struct */
interface UserSettings {
  theme_mode: 'system' | 'dark' | 'light'
  dark_background_opacity: number
  light_background_opacity: number
}

const DEFAULT_SETTINGS: UserSettings = {
  theme_mode: 'system',
  dark_background_opacity: 0.7,
  light_background_opacity: 0.7,
}

/**
 * Determines if dark mode should be active based on theme mode setting
 */
function useThemeMode(themeMode: 'system' | 'dark' | 'light'): boolean {
  const [systemPrefersDark, setSystemPrefersDark] = useState(() => {
    if (globalThis.matchMedia) {
      return globalThis.matchMedia('(prefers-color-scheme: dark)').matches
    }
    return true
  })

  useEffect(() => {
    const mediaQuery = globalThis.matchMedia('(prefers-color-scheme: dark)')
    const handleChange = (e: MediaQueryListEvent) => {
      setSystemPrefersDark(e.matches)
    }
    mediaQuery.addEventListener('change', handleChange)
    return () => mediaQuery.removeEventListener('change', handleChange)
  }, [])

  // Determine actual dark mode based on theme setting
  if (themeMode === 'dark') return true
  if (themeMode === 'light') return false
  return systemPrefersDark // 'system' mode
}

/**
 * Applies background opacity CSS variables based on user settings
 * Opacity: 0.0 = fully transparent, 1.0 = fully opaque
 */
function applyBackgroundOpacity(settings: UserSettings) {
  const root = document.documentElement

  // The gradient end is slightly less opaque than start for a subtle effect
  // Using a small offset (0.03) to create a gentle gradient
  const darkStart = settings.dark_background_opacity
  const darkEnd = darkStart >= 1 ? 1 : Math.max(0, darkStart - 0.03)
  const lightStart = settings.light_background_opacity
  const lightEnd = lightStart >= 1 ? 1 : Math.max(0, lightStart - 0.05)

  root.style.setProperty('--win11-dark-bg-alpha-start', darkStart.toString())
  root.style.setProperty('--win11-dark-bg-alpha-end', darkEnd.toString())
  root.style.setProperty('--win11-light-bg-alpha-start', lightStart.toString())
  root.style.setProperty('--win11-light-bg-alpha-end', lightEnd.toString())
}

/**
 * Updates the document's dark class based on theme
 */
function applyThemeClass(isDark: boolean) {
  if (isDark) {
    document.documentElement.classList.add('dark')
  } else {
    document.documentElement.classList.remove('dark')
  }
}

/**
 * Main Clipboard App Component
 */
function ClipboardApp() {
  const [activeTab, setActiveTab] = useState<ActiveTab>('clipboard')
  const [settings, setSettings] = useState<UserSettings>(DEFAULT_SETTINGS)
  const [settingsLoaded, setSettingsLoaded] = useState(false)
  const [focusedIndex, setFocusedIndex] = useState(0)

  const isDark = useThemeMode(settings.theme_mode)

  const { history, isLoading, clearHistory, deleteItem, togglePin, pasteItem } =
    useClipboardHistory()

  // Refs for focus management
  const tabBarRef = useRef<TabBarRef>(null)
  const historyItemRefs = useRef<(HTMLDivElement | null)[]>([])
  const contentContainerRef = useRef<HTMLDivElement>(null)

  // Load initial settings and set up listener for changes
  useEffect(() => {
    // Load initial settings
    invoke<UserSettings>('get_user_settings')
      .then((loadedSettings) => {
        setSettings(loadedSettings)
        applyBackgroundOpacity(loadedSettings)
        setSettingsLoaded(true)
      })
      .catch((err) => {
        console.error('Failed to load user settings:', err)
        applyBackgroundOpacity(DEFAULT_SETTINGS)
        setSettingsLoaded(true)
      })

    // Listen for settings changes from the settings window
    const unlistenPromise = listen<UserSettings>('app-settings-changed', (event) => {
      const newSettings = event.payload
      setSettings(newSettings)
      applyBackgroundOpacity(newSettings)
    })

    return () => {
      unlistenPromise.then((unlisten) => unlisten())
    }
  }, [])

  // Apply theme class when isDark changes
  useEffect(() => {
    applyThemeClass(isDark)
  }, [isDark])

  // Handle ESC key to close/hide window
  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault()
        try {
          await getCurrentWindow().hide()
        } catch (err) {
          console.error('Failed to hide window:', err)
        }
      }
    }

    globalThis.addEventListener('keydown', handleKeyDown)
    return () => globalThis.removeEventListener('keydown', handleKeyDown)
  }, [])

  // Use refs to store current values for the focus handler (to avoid re-registering listener)
  const activeTabRef = useRef(activeTab)
  const historyRef = useRef(history)

  // Keep refs in sync
  useEffect(() => {
    activeTabRef.current = activeTab
  }, [activeTab])

  useEffect(() => {
    historyRef.current = history
  }, [history])

  // Handle window-shown event for focus management (registered once)
  useEffect(() => {
    const focusFirstItem = () => {
      // Small delay to ensure the window is fully rendered and focused
      setTimeout(() => {
        const currentTab = activeTabRef.current
        const currentHistory = historyRef.current

        if (currentTab === 'clipboard') {
          // Focus the first history item if on clipboard tab
          if (currentHistory.length > 0) {
            setFocusedIndex(0)
            historyItemRefs.current[0]?.focus()
          }
        } else {
          // Focus the first tab button if on other tabs
          tabBarRef.current?.focusFirstTab()
        }
      }, 100)
    }

    // Listen to window-shown event (emitted from Rust when window is toggled visible)
    const unlistenWindowShown = listen('window-shown', focusFirstItem)

    return () => {
      unlistenWindowShown.then((unlisten) => unlisten())
    }
  }, []) // Empty dependency array - listener is registered once

  // Keyboard navigation for clipboard items
  useEffect(() => {
    if (activeTab !== 'clipboard' || history.length === 0) return

    const handleArrowKeys = (e: KeyboardEvent) => {
      // Check if a tab button is focused - if so, don't intercept arrows
      const activeElement = document.activeElement
      if (activeElement?.getAttribute('role') === 'tab') return

      if (e.key === 'ArrowDown') {
        e.preventDefault()
        const newIndex = Math.min(focusedIndex + 1, history.length - 1)
        setFocusedIndex(newIndex)
        historyItemRefs.current[newIndex]?.focus()
        historyItemRefs.current[newIndex]?.scrollIntoView({ block: 'nearest' })
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        const newIndex = Math.max(focusedIndex - 1, 0)
        setFocusedIndex(newIndex)
        historyItemRefs.current[newIndex]?.focus()
        historyItemRefs.current[newIndex]?.scrollIntoView({ block: 'nearest' })
      } else if (e.key === 'Home') {
        e.preventDefault()
        setFocusedIndex(0)
        historyItemRefs.current[0]?.focus()
        historyItemRefs.current[0]?.scrollIntoView({ block: 'nearest' })
      } else if (e.key === 'End') {
        e.preventDefault()
        const lastIndex = history.length - 1
        setFocusedIndex(lastIndex)
        historyItemRefs.current[lastIndex]?.focus()
        historyItemRefs.current[lastIndex]?.scrollIntoView({ block: 'nearest' })
      }
    }

    globalThis.addEventListener('keydown', handleArrowKeys)
    return () => globalThis.removeEventListener('keydown', handleArrowKeys)
  }, [activeTab, focusedIndex, history.length])

  // Handle tab change
  const handleTabChange = useCallback((tab: ActiveTab) => {
    setActiveTab(tab)
    setFocusedIndex(0) // Reset focused index when changing tabs
  }, [])

  const handleMouseEnter = () => {
    invoke('set_mouse_state', { inside: true }).catch(console.error)
  }

  const handleMouseLeave = () => {
    invoke('set_mouse_state', { inside: false }).catch(console.error)
  }

  // Render content based on active tab
  const renderContent = () => {
    switch (activeTab) {
      case 'clipboard':
        if (isLoading) {
          return (
            <div className="flex items-center justify-center h-full">
              <div className="w-6 h-6 border-2 border-win11-bg-accent border-t-transparent rounded-full animate-spin" />
            </div>
          )
        }

        if (history.length === 0) {
          return <EmptyState />
        }

        return (
          <>
            <Header
              onClearHistory={clearHistory}
              itemCount={history.filter((i) => !i.pinned).length}
            />
            <div className="flex flex-col gap-2 p-3">
              {history.map((item, index) => (
                <HistoryItem
                  key={item.id}
                  ref={(el) => {
                    historyItemRefs.current[index] = el
                  }}
                  item={item}
                  index={index}
                  onPaste={pasteItem}
                  onDelete={deleteItem}
                  onTogglePin={togglePin}
                  onFocus={() => setFocusedIndex(index)}
                />
              ))}
            </div>
          </>
        )

      case 'emoji':
        return <EmojiPicker />

      case 'gifs':
        return <GifPicker />

      default:
        return null
    }
  }

  // Don't render until settings are loaded to prevent FOUC
  if (!settingsLoaded) {
    return null
  }

  return (
    <div
      className={clsx(
        'h-screen w-screen overflow-hidden flex flex-col rounded-win11-lg',
        isDark ? 'glass-effect' : 'glass-effect-light',
        isDark ? 'bg-win11-acrylic-bg' : 'bg-win11Light-acrylic-bg',
        isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
      )}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {/* Drag Handle */}
      <DragHandle />

      {/* Tab bar */}
      <TabBar ref={tabBarRef} activeTab={activeTab} onTabChange={handleTabChange} />

      {/* Scrollable content area */}
      <div
        ref={contentContainerRef}
        className={clsx(
          'flex-1',
          // Only use scrollbar for non-emoji tabs, emoji has its own virtualized scrolling
          activeTab === 'emoji' ? 'overflow-hidden' : 'overflow-y-auto scrollbar-win11'
        )}
      >
        {renderContent()}
      </div>
    </div>
  )
}

export default ClipboardApp
