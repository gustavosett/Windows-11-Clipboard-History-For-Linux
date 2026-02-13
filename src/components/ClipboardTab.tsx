import { useState, useMemo, useRef, useEffect, useCallback } from 'react'
import { listen } from '@tauri-apps/api/event'
import { clsx } from 'clsx'

import type { ClipboardItem, UserSettings } from '../types/clipboard'
import type { TabBarRef } from './TabBar'
import { Header } from './Header'
import { SearchBar } from './common/SearchBar'
import { EmptyState } from './EmptyState'
import { HistoryItem } from './HistoryItem'
import { useHistoryKeyboardNavigation } from '../hooks/useHistoryKeyboardNavigation'

export function ClipboardTab(props: {
  history: ClipboardItem[]
  isLoading: boolean
  isDark: boolean
  tertiaryOpacity: number
  secondaryOpacity: number
  clearHistory: () => void
  deleteItem: (id: string) => void
  togglePin: (id: string) => void
  onPaste: (id: string) => void
  settings: UserSettings
  tabBarRef: React.RefObject<TabBarRef | null>
}) {
  const {
    history,
    isLoading,
    isDark,
    tertiaryOpacity,
    secondaryOpacity,
    clearHistory,
    deleteItem,
    togglePin,
    onPaste,
    settings,
    tabBarRef,
  } = props

  const [searchQuery, setSearchQuery] = useState('')
  const [isRegexMode, setIsRegexMode] = useState(false)

  const [isCompact, setIsCompact] = useState(() => {
    if (typeof window !== 'undefined') {
      return localStorage.getItem('clipboard-history-compact-mode') === 'true'
    }
    return false
  })

  useEffect(() => {
    localStorage.setItem('clipboard-history-compact-mode', String(isCompact))
  }, [isCompact])
  const [isSearchVisible, setIsSearchVisible] = useState(false)
  const searchInputRef = useRef<HTMLInputElement>(null)

  const [focusedIndex, setFocusedIndex] = useState(0)

  // Refs
  const historyItemRefs = useRef<(HTMLDivElement | null)[]>([])

  // Check if a key is a printable character that should trigger search
  const isPrintableKey = useCallback((e: KeyboardEvent): boolean => {
    // Skip if any modifier key is pressed (except Shift for uppercase/symbols)
    if (e.ctrlKey || e.altKey || e.metaKey) return false

    // Skip special keys that are handled elsewhere
    const specialKeys = [
      'Tab',
      'Enter',
      'Escape',
      'ArrowUp',
      'ArrowDown',
      'ArrowLeft',
      'ArrowRight',
      'Home',
      'End',
      'PageUp',
      'PageDown',
      'Delete',
      'Backspace',
      'F1',
      'F2',
      'F3',
      'F4',
      'F5',
      'F6',
      'F7',
      'F8',
      'F9',
      'F10',
      'F11',
      'F12',
      'CapsLock',
      'NumLock',
      'ScrollLock',
      'Pause',
      'Insert',
      'PrintScreen',
      'ContextMenu',
      'Shift',
      'Control',
      'Alt',
      'Meta',
    ]
    if (specialKeys.includes(e.key)) return false

    // Accept single printable characters (letters, numbers, symbols)
    return e.key.length === 1
  }, [])

  // Toggle search visibility with Ctrl+F or start typing to filter
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      const activeElement = document.activeElement

      // Global shortcuts that should work regardless of focus
      if (e.ctrlKey && e.key.toLowerCase() === 'f') {
        e.preventDefault()
        setIsSearchVisible((prev) => {
          const newValue = !prev
          if (!newValue) {
            // Clear search when hiding
            setSearchQuery('')
          }
          return newValue
        })
        return
      }

      // Close search with Escape - should work even when search input is focused
      if (e.key.toLowerCase() === 'escape' && isSearchVisible) {
        e.preventDefault()
        setIsSearchVisible(false)
        setSearchQuery('')
        return
      }

      // Skip instant filtering if focus is on an input element (user is already typing in search)
      if (activeElement?.tagName === 'INPUT' || activeElement?.tagName === 'TEXTAREA') return

      // Skip if focus is on a tab button (let tab navigation handle it)
      if (activeElement?.getAttribute('role') === 'tab') return

      // Instant filtering: start typing to activate search
      if (isPrintableKey(e)) {
        e.preventDefault()
        // Show search bar and append the typed character
        if (!isSearchVisible) {
          setIsSearchVisible(true)
          setSearchQuery(e.key)
        } else {
          setSearchQuery((prev) => prev + e.key)
        }
        // Focus will be set by the useEffect that watches isSearchVisible
      }
    },
    [isSearchVisible, isPrintableKey]
  )

  // Listen for Ctrl+F
  useEffect(() => {
    globalThis.addEventListener('keydown', handleKeyDown)
    return () => globalThis.removeEventListener('keydown', handleKeyDown)
  }, [handleKeyDown])

  // Focus search input when it becomes visible
  useEffect(() => {
    if (isSearchVisible && searchInputRef.current) {
      searchInputRef.current.focus()
    }
  }, [isSearchVisible])

  // Reset search when window is shown (app reopened)
  useEffect(() => {
    const resetSearch = () => {
      setIsSearchVisible(false)
      setSearchQuery('')
    }
    const unlistenWindowShown = listen('window-shown', resetSearch)
    return () => {
      unlistenWindowShown.then((u) => u())
    }
  }, [])

  // Filter history
  const filteredHistory = useMemo(() => {
    if (!searchQuery) return history

    let regex: RegExp | null = null
    if (isRegexMode) {
      try {
        regex = new RegExp(searchQuery, 'i')
      } catch (err) {
        console.error('Invalid regex pattern in clipboard search query:', searchQuery, err)
        return []
      }
    }

    return history.filter((item) => {
      let searchableText = ''
      if (item.content.type === 'Text') {
        searchableText = item.content.data
      } else if (item.content.type === 'RichText') {
        searchableText = item.content.data.plain
      } else {
        return false
      }

      if (isRegexMode && regex) {
        return regex.test(searchableText)
      } else if (!isRegexMode) {
        return searchableText.toLowerCase().includes(searchQuery.toLowerCase())
      }
      return false
    })
  }, [history, searchQuery, isRegexMode])

  // Keyboard navigation
  useHistoryKeyboardNavigation({
    activeTab: 'clipboard', // Always 'clipboard' when this component is mounted
    itemsLength: filteredHistory.length,
    focusedIndex,
    setFocusedIndex,
    historyItemRefs,
    tabBarRef,
  })

  // Ref for stable access to filtered history in event listener
  const filteredHistoryRef = useRef(filteredHistory)
  useEffect(() => {
    filteredHistoryRef.current = filteredHistory
  }, [filteredHistory])

  useEffect(() => {
    const focusFirstItem = () => {
      setTimeout(() => {
        if (filteredHistoryRef.current.length > 0) {
          setFocusedIndex(0)
          historyItemRefs.current[0]?.focus()
        }
      }, 100)
    }
    const unlistenWindowShown = listen('window-shown', focusFirstItem)
    return () => {
      unlistenWindowShown.then((u) => u())
    }
  }, [])

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full select-none">
        <div className="w-6 h-6 border-2 border-win11-bg-accent border-t-transparent rounded-full animate-spin" />
      </div>
    )
  }

  if (history.length === 0) {
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
      {/* Search Bar - only visible when Ctrl+F is pressed */}
      {isSearchVisible && (
        <div className="px-3 pb-2 pt-1">
          <SearchBar
            ref={searchInputRef}
            value={searchQuery}
            onChange={setSearchQuery}
            isDark={isDark}
            opacity={secondaryOpacity}
            placeholder="Search history..."
            isRegex={isRegexMode}
            onToggleRegex={() => setIsRegexMode(!isRegexMode)}
            onClear={() => {
              setSearchQuery('')
              setIsSearchVisible(false)
            }}
          />
        </div>
      )}

      {filteredHistory.length === 0 ? (
        <div className="flex flex-col items-center justify-center p-8 text-center opacity-60">
          <p
            className={clsx(
              'text-sm',
              isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
            )}
          >
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
              onPaste={onPaste}
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
}
