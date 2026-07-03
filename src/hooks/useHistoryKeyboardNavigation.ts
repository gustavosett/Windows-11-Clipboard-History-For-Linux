import { useEffect } from 'react'
import type { MutableRefObject, RefObject } from 'react'
import type { ActiveTab } from '../types/clipboard'
import type { TabBarRef } from '../components/TabBar'

export function useHistoryKeyboardNavigation(params: {
  activeTab: ActiveTab
  itemsLength: number
  focusedIndex: number
  setFocusedIndex: (i: number) => void
  historyItemRefs: MutableRefObject<(HTMLElement | null)[]>
  tabBarRef: RefObject<TabBarRef | null>
  onUpFromFirstItem?: () => boolean
  onLeftArrow?: () => void
  searchInputRef: RefObject<HTMLInputElement | null>
}) {
  const {
    activeTab,
    itemsLength,
    focusedIndex,
    setFocusedIndex,
    historyItemRefs,
    tabBarRef,
    onUpFromFirstItem,
    onLeftArrow,
    searchInputRef,
  } = params

  useEffect(() => {
    if (activeTab !== 'clipboard' || itemsLength === 0) return

    const handleArrowKeys = (e: KeyboardEvent) => {
      const activeElement = document.activeElement
      if (activeElement?.getAttribute('role') === 'tab') return
      if (activeElement?.tagName === 'INPUT' && activeElement !== searchInputRef.current) return

      // Check if focus is on a history item, body, or search input
      const isOnHistoryItem =
        historyItemRefs.current.some((ref) => ref === activeElement) ||
        activeElement === document.body
      const isOnSearchInput = activeElement === searchInputRef.current
      if (!isOnHistoryItem && !isOnSearchInput) return

      if (e.key === 'ArrowDown') {
        e.preventDefault()
        const newIndex = isOnSearchInput ? 0 : Math.min(focusedIndex + 1, itemsLength - 1)
        setFocusedIndex(newIndex)
        historyItemRefs.current[newIndex]?.focus()
        historyItemRefs.current[newIndex]?.scrollIntoView({ block: 'nearest' })
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        if (isOnSearchInput) return
        if (focusedIndex === 0 && onUpFromFirstItem?.()) return
        if (focusedIndex === 0) {
          searchInputRef.current?.focus()
          return
        }
        const newIndex = Math.max(focusedIndex - 1, 0)
        setFocusedIndex(newIndex)
        historyItemRefs.current[newIndex]?.focus()
        historyItemRefs.current[newIndex]?.scrollIntoView({ block: 'nearest' })
      } else if (e.key === 'ArrowLeft') {
        if (onLeftArrow) {
          e.preventDefault()
          onLeftArrow()
        }
      } else if (e.key === 'Home') {
        e.preventDefault()
        setFocusedIndex(0)
        historyItemRefs.current[0]?.focus()
        historyItemRefs.current[0]?.scrollIntoView({ block: 'nearest' })
      } else if (e.key === 'End') {
        e.preventDefault()
        const lastIndex = itemsLength - 1
        setFocusedIndex(lastIndex)
        historyItemRefs.current[lastIndex]?.focus()
        historyItemRefs.current[lastIndex]?.scrollIntoView({ block: 'nearest' })
      } else if (e.key === 'Tab' && !e.shiftKey) {
        e.preventDefault()
        tabBarRef.current?.focusFirstTab()
      }
    }

    globalThis.addEventListener('keydown', handleArrowKeys)
    return () => globalThis.removeEventListener('keydown', handleArrowKeys)
  }, [
    activeTab,
    itemsLength,
    focusedIndex,
    setFocusedIndex,
    historyItemRefs,
    tabBarRef,
    onUpFromFirstItem,
    onLeftArrow,
    searchInputRef,
  ])
}
