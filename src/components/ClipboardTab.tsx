import { useState, useMemo, useRef, useEffect, useCallback } from 'react'
import { listen } from '@tauri-apps/api/event'
import { clsx } from 'clsx'

import type { ClipboardItem, UserSettings } from '../types/clipboard'
import type { TabBarRef } from './TabBar'
import { Header } from './Header'
import { SearchBar } from './SearchBar'
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

  // Toggle search visibility with Ctrl+F
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === 'f') {
        e.preventDefault()
        setIsSearchVisible((prev) => {
          const newValue = !prev
          if (!newValue) {
            // Clear search when hiding
            setSearchQuery('')
          }
          return newValue
        })
      }
      // Close search with Escape
      if (e.key === 'Escape' && isSearchVisible) {
        e.preventDefault()
        setIsSearchVisible(false)
        setSearchQuery('')
      }
    },
    [isSearchVisible]
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
      if (item.content.type !== 'Text') return false

      if (isRegexMode && regex) {
        return regex.test(item.content.data)
      } else if (!isRegexMode) {
        return item.content.data.toLowerCase().includes(searchQuery.toLowerCase())
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
