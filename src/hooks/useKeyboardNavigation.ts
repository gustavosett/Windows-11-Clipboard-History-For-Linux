import { useCallback } from 'react'

interface UseKeyboardNavigationProps<T> {
  items: T[]
  columnCount: number
  onSelect: (item: T) => void
  setFocusedIndex: (index: number) => void
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  gridRef?: React.RefObject<any>
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  containerRef?: React.RefObject<any>
  dataAttributeName?: string
}

export function useKeyboardNavigation<T>({
  items,
  columnCount,
  onSelect,
  setFocusedIndex,
  gridRef,
  containerRef,
  dataAttributeName,
}: UseKeyboardNavigationProps<T>) {
  return useCallback(
    (e: React.KeyboardEvent, currentIndex: number) => {
      if (!items || items.length === 0) return

      let newIndex = currentIndex
      let handled = false

      switch (e.key) {
        case 'ArrowRight':
          if (currentIndex < items.length - 1) {
            newIndex = currentIndex + 1
            handled = true
          }
          break
        case 'ArrowLeft':
          if (currentIndex > 0) {
            newIndex = currentIndex - 1
            handled = true
          }
          break
        case 'ArrowDown': {
          const nextRowIndex = currentIndex + columnCount
          if (nextRowIndex < items.length) {
            newIndex = nextRowIndex
            handled = true
          }
          break
        }
        case 'ArrowUp': {
          const prevRowIndex = currentIndex - columnCount
          if (prevRowIndex >= 0) {
            newIndex = prevRowIndex
            handled = true
          }
          break
        }
        case 'Home':
          if (e.ctrlKey) {
            newIndex = 0
          } else {
            const currentRow = Math.floor(currentIndex / columnCount)
            newIndex = currentRow * columnCount
          }
          handled = true
          break
        case 'End':
          if (e.ctrlKey) {
            newIndex = items.length - 1
          } else {
            const currentRow = Math.floor(currentIndex / columnCount)
            newIndex = Math.min((currentRow + 1) * columnCount - 1, items.length - 1)
          }
          handled = true
          break
        case 'PageDown':
          newIndex = Math.min(currentIndex + columnCount * 3, items.length - 1)
          handled = true
          break
        case 'PageUp':
          newIndex = Math.max(currentIndex - columnCount * 3, 0)
          handled = true
          break
        case 'Enter':
        case ' ':
          e.preventDefault()
          if (items[currentIndex]) {
            onSelect(items[currentIndex])
          }
          return
      }

      if (handled) {
        e.preventDefault()
        e.stopPropagation()
        setFocusedIndex(newIndex)

        if (gridRef?.current?.scrollToCell) {
          const targetRow = Math.floor(newIndex / columnCount)
          const targetCol = newIndex % columnCount
          gridRef.current.scrollToCell({
            rowIndex: targetRow,
            columnIndex: targetCol,
            rowAlign: 'smart',
            columnAlign: 'smart',
          })
        }

        if (containerRef?.current && dataAttributeName) {
          setTimeout(() => {
            const element = containerRef.current?.querySelector(
              `[${dataAttributeName}="${newIndex}"]`
            ) as HTMLElement
            element?.focus()
          }, 10)
        }
      }
    },
    [items, columnCount, onSelect, setFocusedIndex, gridRef, containerRef, dataAttributeName]
  )
}
