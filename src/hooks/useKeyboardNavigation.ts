import { useCallback } from 'react'

/**
 * Minimal structural types for the refs this hook touches.
 *
 * The consumers pass refs to their concrete elements (react-window `Grid`,
 * plain DOM containers). Instead of `any`, the hook narrows the ref targets
 * to these interfaces before use — so every member access below is typed.
 *
 * تایپ‌های ساختاری کمینه برای refهایی که این هوک لمس می‌کند.
 *
 * مصرف‌کنندگان ref به عناصر واقعی خودشان (react-window Grid، کانتینر DOM)
 * پاس می‌دهند. به‌جای `any`، هوک هدف ref را پیش از استفاده به این
 * interfaceها محدود می‌کند تا هر دسترسی عضو در پایین تایپ‌دار باشد.
 */
interface GridScrollTarget {
  scrollToCell(options: {
    rowIndex: number
    columnIndex: number
    rowAlign?: string
    columnAlign?: string
  }): void
}

interface DomContainerTarget {
  querySelector(selector: string): HTMLElement | null
}

interface UseKeyboardNavigationProps<T> {
  items: T[]
  columnCount: number
  onSelect: (item: T) => void
  setFocusedIndex: (index: number) => void
  // The concrete element types live in the consumers (react-window Grid /
  // DOM containers); `unknown` keeps this hook decoupled from react-window.
  // نوع عنصر واقعی نزد مصرف‌کننده است (react-window Grid / کانتینر DOM)؛
  // `unknown` این هوک را از react-window جدا نگه می‌دارد.
  gridRef?: React.RefObject<unknown>
  containerRef?: React.RefObject<unknown>
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

        // Narrow `unknown` to the structural target before use; optional
        // chaining keeps this a no-op for refs that do not provide the API.
        // `unknown` را پیش از استفاده به هدف ساختاری محدود می‌کنیم؛ optional
        // chaining برای refهایی که API را ندارند، عملیات را بی‌اثر می‌کند.
        const grid = gridRef?.current as GridScrollTarget | null | undefined
        if (grid?.scrollToCell) {
          const targetRow = Math.floor(newIndex / columnCount)
          const targetCol = newIndex % columnCount
          grid.scrollToCell({
            rowIndex: targetRow,
            columnIndex: targetCol,
            rowAlign: 'smart',
            columnAlign: 'smart',
          })
        }

        if (containerRef?.current && dataAttributeName) {
          const container = containerRef.current as DomContainerTarget
          setTimeout(() => {
            const element = container.querySelector(
              `[${dataAttributeName}="${newIndex}"]`
            )
            element?.focus()
          }, 10)
        }
      }
    },
    [items, columnCount, onSelect, setFocusedIndex, gridRef, containerRef, dataAttributeName]
  )
}
