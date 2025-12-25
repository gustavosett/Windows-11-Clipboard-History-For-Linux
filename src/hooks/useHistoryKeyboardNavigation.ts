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
}) {
  const {
    activeTab,
    itemsLength,
    focusedIndex,
    setFocusedIndex,
    historyItemRefs,
    tabBarRef,
  } = params

  useEffect(() => {
    if (activeTab !== 'clipboard' || itemsLength === 0) return

    const handleArrowKeys = (e: KeyboardEvent) => {
      // Check if a tab button is focused - if so, don't intercept arrows
      const activeElement = document.activeElement
      if (activeElement?.getAttribute('role') === 'tab') return
      // Check if search bar is focused
      if (activeElement?.tagName === 'INPUT') return

      // Check if focus is on a history item or body
      const isOnHistoryItem =
        historyItemRefs.current.some((ref) => ref === activeElement) ||
        activeElement === document.body
      if (!isOnHistoryItem) return

      if (e.key === 'ArrowDown') {
        e.preventDefault()
        const newIndex = Math.min(focusedIndex + 1, itemsLength - 1)
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
        const lastIndex = itemsLength - 1
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
  }, [activeTab, itemsLength, focusedIndex, setFocusedIndex, historyItemRefs, tabBarRef])
}
