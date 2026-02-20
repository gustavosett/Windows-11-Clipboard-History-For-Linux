import { useEffect, useLayoutEffect, useRef } from 'react'
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
  const { activeTab, itemsLength, focusedIndex, setFocusedIndex, historyItemRefs, tabBarRef } =
    params

  // Sync mutable refs so the stable listener can always read the latest values
  // without being re-registered on every navigation step or filter change.
  const focusedIndexRef = useRef(focusedIndex)
  const itemsLengthRef = useRef(itemsLength)

  useLayoutEffect(() => {
    focusedIndexRef.current = focusedIndex
    itemsLengthRef.current = itemsLength
  })

  useEffect(() => {
    const handleArrowKeys = (e: KeyboardEvent) => {
      // Guard moved inside the handler so the listener is always
      // registered and picks up future items without relying on dep-array churn.
      if (activeTab !== 'clipboard' || itemsLengthRef.current === 0) return

      const activeElement = document.activeElement

      // Don't intercept when a tab button is focused
      if (activeElement?.getAttribute('role') === 'tab') return

      // Only act when focus is on a history item, the body, or the search input
      const isOnHistoryItem =
        historyItemRefs.current.some((ref) => ref === activeElement) ||
        activeElement === document.body ||
        activeElement?.tagName === 'INPUT'
      if (!isOnHistoryItem) return

      // Let the browser handle cursor-movement keys inside input fields
      if (activeElement?.tagName === 'INPUT' && (e.key === 'Home' || e.key === 'End')) return

      const currentIndex = focusedIndexRef.current
      const length = itemsLengthRef.current

      if (e.key === 'ArrowDown') {
        e.preventDefault()
        const newIndex = Math.min(currentIndex + 1, length - 1)
        setFocusedIndex(newIndex)
        historyItemRefs.current[newIndex]?.focus()
        historyItemRefs.current[newIndex]?.scrollIntoView({ block: 'nearest' })
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        const newIndex = Math.max(currentIndex - 1, 0)
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
        const lastIndex = length - 1
        setFocusedIndex(lastIndex)
        historyItemRefs.current[lastIndex]?.focus()
        historyItemRefs.current[lastIndex]?.scrollIntoView({ block: 'nearest' })
      } else if (e.key === 'Tab' && !e.shiftKey) {
        // Only intercept Tab when a history item div is focused, not when an
        // input element is focused (search bar should keep normal Tab behaviour)
        const isOnActualHistoryItem = historyItemRefs.current.some((ref) => ref === activeElement)
        if (!isOnActualHistoryItem) return
        e.preventDefault()
        tabBarRef.current?.focusFirstTab()
      }
    }

    globalThis.addEventListener('keydown', handleArrowKeys)
    return () => globalThis.removeEventListener('keydown', handleArrowKeys)
  }, [activeTab, setFocusedIndex, historyItemRefs, tabBarRef])
}
