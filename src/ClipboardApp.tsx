import { useState, useCallback, useEffect, useRef, useMemo } from 'react'
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
import { SearchBar } from './components/SearchBar'
import { Toast } from './components/Toast'
import { KaomojiPicker } from './components/KaomojiPicker'
import { calculateSecondaryOpacity, calculateTertiaryOpacity } from './utils/themeUtils'
import type { ActiveTab, UserSettings } from './types/clipboard'

const DEFAULT_SETTINGS: UserSettings = {
  theme_mode: 'system',
  dark_background_opacity: 0.7,
  light_background_opacity: 0.7,
  enable_smart_actions: true,

  enable_ui_polish: true,
  custom_kaomojis: [],
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

  // Search State
  const [searchQuery, setSearchQuery] = useState('')
  const [isRegexMode, setIsRegexMode] = useState(false)
  
  // UI States
  const [isCompact, setIsCompact] = useState(false)
  const [toastMessage, setToastMessage] = useState<string | null>(null)

  const isDark = useThemeMode(settings.theme_mode)
  const opacity = isDark ? settings.dark_background_opacity : settings.light_background_opacity
  const secondaryOpacity = calculateSecondaryOpacity(opacity)
  const tertiaryOpacity = calculateTertiaryOpacity(opacity)

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

  // Filter history based on search query and active tab
  const filteredHistory = useMemo(() => {
    const items = history



    if (!searchQuery) return items

    return items.filter((item) => {
      if (item.content.type !== 'Text') return false
      
      try {
        if (isRegexMode) {
            const regex = new RegExp(searchQuery, 'i')
            return regex.test(item.content.data)
        } else {
            return item.content.data.toLowerCase().includes(searchQuery.toLowerCase())
        }
      } catch {
        // Invalid regex
        return false
      }
    })
  }, [history, searchQuery, isRegexMode])

  // Use refs to store current values for the focus handler (to avoid re-registering listener)
  const activeTabRef = useRef(activeTab)
  const historyRef = useRef(filteredHistory) // Use filtered history

  // Keep refs in sync
  useEffect(() => {
    activeTabRef.current = activeTab
  }, [activeTab])

  useEffect(() => {
    historyRef.current = filteredHistory
  }, [filteredHistory])

  // Handle window-shown event for focus management (registered once)
  useEffect(() => {
    const focusFirstItem = () => {
      // Small delay to ensure the window is fully rendered and focused
      setTimeout(() => {
        const currentTab = activeTabRef.current
        const currentHistory = historyRef.current

        if (currentTab === 'clipboard') {
          // Focus the first history item if on clipboard or favorites tab
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
    if (activeTab !== 'clipboard' || filteredHistory.length === 0) return

    const handleArrowKeys = (e: KeyboardEvent) => {
      // Check if a tab button is focused - if so, don't intercept arrows
      const activeElement = document.activeElement
      if (activeElement?.getAttribute('role') === 'tab') return
      // Check if search bar is focused
      if (activeElement?.tagName === 'INPUT') return

      // Check if focus is on a history item
      if (historyItemRefs.current.some((ref) => ref === activeElement) || activeElement === document.body) {
         // Allow navigation if focus is on body (just opened) or history item
      } else {
         return
      }

      if (e.key === 'ArrowDown') {
        e.preventDefault()
        const newIndex = Math.min(focusedIndex + 1, filteredHistory.length - 1)
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
        const lastIndex = filteredHistory.length - 1
        setFocusedIndex(lastIndex)
        historyItemRefs.current[lastIndex]?.focus()
        historyItemRefs.current[lastIndex]?.scrollIntoView({ block: 'nearest' })
      } else if (e.key === 'Tab' && !e.shiftKey) {
        // When pressing Tab on a history item, go back to the tab bar
        e.preventDefault()
        tabBarRef.current?.focusFirstTab()
      }
    }

    globalThis.addEventListener('keydown', handleArrowKeys)
    return () => globalThis.removeEventListener('keydown', handleArrowKeys)
  }, [activeTab, focusedIndex, filteredHistory.length])

  // Handle tab change
  const handleTabChange = useCallback((tab: ActiveTab) => {
    setActiveTab(tab)
    setFocusedIndex(0) // Reset focused index when changing tabs
    setSearchQuery('') // Clear search
  }, [])

  const handleMouseEnter = () => {
    invoke('set_mouse_state', { inside: true }).catch(console.error)
  }

  const handleMouseLeave = () => {
    invoke('set_mouse_state', { inside: false }).catch(console.error)
  }
  
  // Show toast helper
  const showToast = useCallback((msg: string) => {
      setToastMessage(msg)
  }, [])
  
  // Override paste to show toast
  const handlePaste = useCallback((id: string) => {
      pasteItem(id)
      if (settings.enable_ui_polish) {
          showToast('Copied to clipboard!')
      }
  }, [pasteItem, showToast, settings.enable_ui_polish])



  // Render content based on active tab
  const renderContent = () => {
    switch (activeTab) {
      case 'clipboard':

        if (isLoading) {
          return (
            <div className="flex items-center justify-center h-full select-none">
              <div className="w-6 h-6 border-2 border-win11-bg-accent border-t-transparent rounded-full animate-spin" />
            </div>
          )
        }

        // Different empty states could be nice, but sharing for now
        if (history.length === 0 && activeTab === 'clipboard') {
          return <EmptyState isDark={isDark} />
        }
        


        return (
          <>
            <Header
              onClearHistory={clearHistory}
              itemCount={filteredHistory.length}
              isDark={isDark}
              tertiaryOpacity={tertiaryOpacity}
              isCompact={isCompact}
              onToggleCompact={() => setIsCompact(!isCompact)}
            />
            {/* Search Bar for Clipboard & Favorites */}
            <div className="px-3 pb-2 pt-1">
                 <SearchBar
                    value={searchQuery}
                    onChange={setSearchQuery}
                    isDark={isDark}
                    opacity={secondaryOpacity}
                    placeholder="Search history..."
                    isRegex={isRegexMode}
                    onToggleRegex={() => setIsRegexMode(!isRegexMode)}
                    onClear={() => setSearchQuery('')}
                 />
            </div>
            
            {filteredHistory.length === 0 ? (
                <div className="flex flex-col items-center justify-center p-8 text-center opacity-60">
                    <p className={clsx("text-sm", isDark ? "text-win11-text-secondary" : "text-win11Light-text-secondary")}>
                        No items found
                    </p>
                </div>
            ) : (
                <div className="flex flex-col gap-2 p-3" role="listbox" aria-label="Clipboard history">
                  {filteredHistory.map((item, index) => (
                    <HistoryItem
                      key={item.id}
                      ref={(el) => {
                        historyItemRefs.current[index] = el
                      }}
                      item={item}
                      index={index}
                      isFocused={index === focusedIndex}
                      onPaste={handlePaste}
                      onDelete={deleteItem}
                      onTogglePin={togglePin}
                      onFocus={() => setFocusedIndex(index)}
                      isDark={isDark}
                      secondaryOpacity={secondaryOpacity}
                      isCompact={isCompact}
                      enableSmartActions={settings.enable_smart_actions}
                      enableUiPolish={settings.enable_ui_polish}
                    />
                  ))}
                </div>
            )}
          </>
        )

      case 'emoji':
        return <EmojiPicker isDark={isDark} opacity={secondaryOpacity} />

      case 'gifs':
        return <GifPicker isDark={isDark} opacity={secondaryOpacity} />

      case 'kaomoji':
        return <KaomojiPicker isDark={isDark} opacity={secondaryOpacity} onShowToast={showToast} customKaomojis={settings.custom_kaomojis} />

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
        'h-screen w-screen overflow-hidden flex flex-col rounded-win11-lg select-none',
        isDark ? 'glass-effect' : 'glass-effect-light',
        isDark ? 'bg-win11-acrylic-bg' : 'bg-win11Light-acrylic-bg',
        isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
      )}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {/* Toast Notification */}
      {settings.enable_ui_polish && (
          <Toast 
              message={toastMessage} 
              onClose={() => setToastMessage(null)} 
              isDark={isDark} 
              opacity={secondaryOpacity} 
          />
      )}

      {/* Drag Handle */}
      <DragHandle isDark={isDark} />

      {/* Tab bar */}
      <TabBar
        ref={tabBarRef}
        activeTab={activeTab}
        onTabChange={handleTabChange}
        isDark={isDark}
        tertiaryOpacity={tertiaryOpacity}

      />

      {/* Scrollable content area */}
      <div
        ref={contentContainerRef}
        className={clsx(
          'flex-1',
          // Only use scrollbar for non-emoji/gif/kaomoji tabs, they have their own virtualized scrolling or containers
          activeTab === 'emoji' || activeTab === 'gifs' || activeTab === 'kaomoji'
            ? 'overflow-hidden'
            : 'overflow-y-auto scrollbar-win11'
        )}
      >
        {renderContent()}
      </div>
    </div>
  )
}

export default ClipboardApp
