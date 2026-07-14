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
  searchInputRef: RefObject<HTMLInputElement | null>
  onUpFromFirstItem?: () => boolean
  onLeftArrow?: () => void
}) {
  const {
    activeTab,
    itemsLength,
    focusedIndex,
    setFocusedIndex,
    historyItemRefs,
    tabBarRef,
    searchInputRef,
    onUpFromFirstItem,
    onLeftArrow,
  } = params

  useEffect(() => {
    if (activeTab !== 'clipboard' || itemsLength === 0) return

    const handleArrowKeys = (e: KeyboardEvent) => {
      const activeElement = document.activeElement
      if (activeElement?.getAttribute('role') === 'tab') return

      const isOnHistoryItem =
        historyItemRefs.current.some((ref) => ref === activeElement) ||
        activeElement === document.body
      const isOnSearchInput = activeElement === searchInputRef.current
      if (activeElement?.tagName === 'INPUT' && !isOnSearchInput) return
      if (!isOnHistoryItem && !isOnSearchInput) return
      if (isOnSearchInput && e.key !== 'ArrowDown' && e.key !== 'ArrowUp') return

      if (e.key === 'ArrowDown') {
        e.preventDefault()
        const newIndex = isOnSearchInput ? 0 : Math.min(focusedIndex + 1, itemsLength - 1)
        setFocusedIndex(newIndex)
        historyItemRefs.current[newIndex]?.focus()
        historyItemRefs.current[newIndex]?.scrollIntoView({ block: 'nearest' })
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        if (isOnSearchInput) return
        if (focusedIndex === 0) {
          if (onUpFromFirstItem?.()) return
          searchInputRef.current?.focus()
          return
        }
        const newIndex = Math.max(focusedIndex - 1, 0)
        setFocusedIndex(newIndex)
        historyItemRefs.current[newIndex]?.focus()
        historyItemRefs.current[newIndex]?.scrollIntoView({ block: 'nearest' })
      } else if (e.key === 'ArrowLeft') {
        if (onLeftArrow && !isOnSearchInput) {
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
    searchInputRef,
    onUpFromFirstItem,
    onLeftArrow,
  ])
}
