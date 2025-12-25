import { useState, useMemo, useRef, useEffect } from 'react'
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

    tabBarRef,
  } = props

  const [searchQuery, setSearchQuery] = useState('')

  


  const [focusedIndex, setFocusedIndex] = useState(0)

  // Refs
  const historyItemRefs = useRef<(HTMLDivElement | null)[]>([])
  
  // Filter history
  const filteredHistory = useMemo(() => {
    if (!searchQuery) return history

    return history.filter((item) => {
      if (item.content.type !== 'Text') return false
      return item.content.data.toLowerCase().includes(searchQuery.toLowerCase())
    })
  }, [history, searchQuery])

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
       return () => { unlistenWindowShown.then(u => u()) }
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
      />
      {/* Search Bar for Clipboard & Favorites */}
      <div className="px-3 pb-2 pt-1">
           <SearchBar
              value={searchQuery}
              onChange={setSearchQuery}
              isDark={isDark}
              opacity={secondaryOpacity}
              placeholder="Search history..."
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
                onPaste={onPaste}
                onDelete={deleteItem}
                onTogglePin={togglePin}
                onFocus={() => setFocusedIndex(index)}
                isDark={isDark}
                secondaryOpacity={secondaryOpacity}

              />
            ))}
          </div>
      )}
    </>
  )
}
