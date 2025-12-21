/**
 * Emoji Picker Component
 * Windows 11 style emoji picker with virtualized grid for performance
 */
import { useState, useCallback, memo, useRef, useLayoutEffect, useEffect } from 'react'
import { Grid, useGridRef } from 'react-window'
import { clsx } from 'clsx'
import { Clock, ChevronLeft, ChevronRight } from 'lucide-react'
import { useEmojiPicker } from '../hooks/useEmojiPicker'
import { SearchBar } from './SearchBar'
import { getTertiaryBackgroundStyle } from '../utils/themeUtils'
import type { Emoji } from '../services/emojiService'

/** Size of each emoji cell */
const CELL_SIZE = 40
/** Padding inside the grid container */
const GRID_PADDING = 12

interface EmojiCellProps {
  emoji: Emoji
  onSelect: (emoji: Emoji) => void
  onHover?: (emoji: Emoji | null) => void
  tabIndex?: number
  'data-main-index'?: number
  onKeyDown?: (e: React.KeyboardEvent) => void
  onItemFocus?: () => void
}

/** Individual emoji cell - memoized for performance */
const EmojiCell = memo(function EmojiCell({
  emoji,
  onSelect,
  onHover,
  tabIndex = -1,
  'data-main-index': mainIndex,
  onKeyDown,
  onItemFocus,
}: EmojiCellProps) {
  return (
    <button
      onClick={() => onSelect(emoji)}
      onMouseEnter={() => onHover?.(emoji)}
      onMouseLeave={() => onHover?.(null)}
      onFocus={onItemFocus}
      onKeyDown={onKeyDown}
      tabIndex={tabIndex}
      data-main-index={mainIndex}
      className={clsx(
        'flex items-center justify-center',
        'w-full h-full text-2xl',
        'rounded-md transition-transform duration-100',
        'hover:bg-win11Light-bg-tertiary dark:hover:bg-win11-bg-card-hover',
        'hover:scale-110 transform-gpu hover:will-change-transform',
        'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent'
      )}
      title={emoji.name}
      aria-label={emoji.name}
    >
      {emoji.char}
    </button>
  )
})

/** Category pill button */
interface CategoryPillProps {
  category: string
  isActive: boolean
  onClick: () => void
  tabIndex?: number
  onKeyDown?: (e: React.KeyboardEvent) => void
  onFocus?: () => void
  'data-category-index'?: number
  isDark: boolean
  opacity: number
}

const CategoryPill = memo(function CategoryPill({
  category,
  isActive,
  onClick,
  tabIndex = 0,
  onKeyDown,
  onFocus,
  'data-category-index': index,
  isDark,
  opacity,
}: CategoryPillProps) {
  return (
    <button
      onClick={onClick}
      tabIndex={tabIndex}
      onKeyDown={onKeyDown}
      onFocus={onFocus}
      data-category-index={index}
      className={clsx(
        'px-3 py-1 text-xs rounded-full whitespace-nowrap',
        'transition-colors duration-150',
        'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent',
        isActive
          ? 'bg-win11-bg-accent text-white'
          : [
              'text-win11Light-text-secondary dark:text-win11-text-secondary',
              'hover:dark:bg-win11-bg-card-hover hover:bg-win11Light-bg-card-hover',
            ]
      )}
      style={!isActive ? getTertiaryBackgroundStyle(isDark, opacity) : undefined}
    >
      {category}
    </button>
  )
})

interface EmojiGridData {
  emojis: Emoji[]
  onSelect: (emoji: Emoji) => void
  onHover: (emoji: Emoji | null) => void
  focusedIndex: number
  onKeyDown: (e: React.KeyboardEvent, index: number) => void
  onItemFocus: (index: number) => void
  columnCount: number
  columnWidth: number
}

function EmojiGridCell({
  columnIndex,
  rowIndex,
  style,
  emojis,
  onSelect,
  onHover,
  focusedIndex,
  onKeyDown,
  onItemFocus,
  columnCount,
  columnWidth,
  ariaAttributes,
}: {
  columnIndex: number
  rowIndex: number
  style: React.CSSProperties
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ariaAttributes: any
} & EmojiGridData) {
  const index = rowIndex * columnCount + columnIndex
  if (index >= emojis.length) {
    return <></>
  }

  const emoji = emojis[index]
  const isFocused = index === focusedIndex

  return (
    <div
      {...ariaAttributes}
      style={{
        ...style,
        left: Number(style.left) + GRID_PADDING,
        width: columnWidth,
        height: CELL_SIZE,
        padding: 4,
      }}
    >
      <EmojiCell
        emoji={emoji}
        onSelect={onSelect}
        onHover={onHover}
        tabIndex={isFocused ? 0 : -1}
        data-main-index={index}
        onKeyDown={(e) => onKeyDown(e, index)}
        onItemFocus={() => onItemFocus(index)}
      />
    </div>
  )
}

export interface EmojiPickerProps {
  isDark: boolean
  opacity: number
}

export function EmojiPicker({ isDark, opacity }: EmojiPickerProps) {
  const {
    searchQuery,
    setSearchQuery,
    selectedCategory,
    setSelectedCategory,
    categories,
    filteredEmojis,
    recentEmojis,
    isLoading,
    pasteEmoji,
  } = useEmojiPicker()

  const [hoveredEmoji, setHoveredEmoji] = useState<Emoji | null>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const scrollContainerRef = useRef<HTMLDivElement>(null)
  const gridRef = useGridRef(null)
  const recentGridRef = useRef<HTMLDivElement>(null)
  const mainGridContainerRef = useRef<HTMLDivElement>(null)
  const [dimensions, setDimensions] = useState({ width: 0, height: 0 })

  // Roving tabindex state for recent emojis
  const [recentFocusedIndex, setRecentFocusedIndex] = useState(0)
  // Roving tabindex state for main emoji grid
  const [mainFocusedIndex, setMainFocusedIndex] = useState(0)
  // Roving tabindex state for categories
  const [categoryFocusedIndex, setCategoryFocusedIndex] = useState(0)

  // Reset focus indices when emojis change
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setRecentFocusedIndex(0)
    setMainFocusedIndex(0)
  }, [searchQuery, selectedCategory])

  // Measure container size - use useLayoutEffect for synchronous measurement
  useLayoutEffect(() => {
    const updateSize = () => {
      if (containerRef.current) {
        const { width, height } = containerRef.current.getBoundingClientRect()
        if (width > 0 && height > 0) {
          setDimensions((prev) => {
            // Only update if changed to avoid unnecessary re-renders
            if (prev.width !== width || prev.height !== height) {
              return { width, height }
            }
            return prev
          })
        }
      }
    }

    // Initial measurement
    updateSize()

    // Fallback: if ref exists but dimensions weren't captured, retry after paint
    const rafId = requestAnimationFrame(updateSize)

    // Observe for size changes
    const observer = new ResizeObserver(updateSize)
    if (containerRef.current) {
      observer.observe(containerRef.current)
    }

    return () => {
      cancelAnimationFrame(rafId)
      observer.disconnect()
    }
  }, [])

  // Scroll categories
  const scrollCategories = (direction: 'left' | 'right') => {
    if (scrollContainerRef.current) {
      const scrollAmount = 200
      scrollContainerRef.current.scrollBy({
        left: direction === 'left' ? -scrollAmount : scrollAmount,
        behavior: 'smooth',
      })
    }
  }

  // Handle emoji selection
  const handleSelect = useCallback(
    (emoji: Emoji) => {
      pasteEmoji(emoji)
    },
    [pasteEmoji]
  )

  // Calculate grid dimensions based on container
  // Width: use full container width minus padding
  const innerWidth = Math.max(0, dimensions.width - GRID_PADDING * 2)
  const columnCount = Math.max(1, Math.floor(innerWidth / CELL_SIZE))
  const columnWidth = columnCount > 0 ? innerWidth / columnCount : CELL_SIZE
  const rowCount = Math.ceil(filteredEmojis.length / columnCount)

  // Height: use container height, the grid will handle scrolling internally
  const gridHeight = dimensions.height > 0 ? dimensions.height : 200
  const gridWidth = dimensions.width > 0 ? dimensions.width : 320

  // Recent emojis column count (they're smaller)
  const recentColumnCount = 8

  // Keyboard navigation for categories
  const handleCategoryKeyDown = useCallback(
    (e: React.KeyboardEvent, currentIndex: number) => {
      // Total items = 1 (All) + categories.length
      const totalItems = 1 + categories.length
      let newIndex = currentIndex
      let handled = false

      switch (e.key) {
        case 'ArrowRight':
          if (currentIndex < totalItems - 1) {
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
        case 'Home':
          newIndex = 0
          handled = true
          break
        case 'End':
          newIndex = totalItems - 1
          handled = true
          break
        case 'Enter':
        case ' ':
          e.preventDefault()
          if (currentIndex === 0) {
            setSelectedCategory(null)
          } else {
            setSelectedCategory(categories[currentIndex - 1])
          }
          return
      }

      if (handled) {
        e.preventDefault()
        e.stopPropagation()
        setCategoryFocusedIndex(newIndex)

        // Scroll container if needed
        const container = scrollContainerRef.current
        if (container) {
          const button = container.querySelector(
            `[data-category-index="${newIndex}"]`
          ) as HTMLElement
          if (button) {
            button.focus()
            button.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' })
          }
        }
      }
    },
    [categories, setSelectedCategory]
  )

  // Keyboard navigation for recent emojis grid
  const handleRecentKeyDown = useCallback(
    (e: React.KeyboardEvent, currentIndex: number) => {
      const recentCount = Math.min(recentEmojis.length, 16)
      if (recentCount === 0) return

      let newIndex = currentIndex
      let handled = false

      switch (e.key) {
        case 'ArrowRight':
          if (currentIndex < recentCount - 1) {
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
          const nextRowIndex = currentIndex + recentColumnCount
          if (nextRowIndex < recentCount) {
            newIndex = nextRowIndex
            handled = true
          }
          break
        }
        case 'ArrowUp': {
          const prevRowIndex = currentIndex - recentColumnCount
          if (prevRowIndex >= 0) {
            newIndex = prevRowIndex
            handled = true
          }
          break
        }
        case 'Home':
          newIndex = 0
          handled = true
          break
        case 'End':
          newIndex = recentCount - 1
          handled = true
          break
        case 'Enter':
        case ' ':
          e.preventDefault()
          if (recentEmojis[currentIndex]) {
            handleSelect(recentEmojis[currentIndex])
          }
          return
      }

      if (handled) {
        e.preventDefault()
        e.stopPropagation()
        setRecentFocusedIndex(newIndex)
        // Focus the new element
        const container = recentGridRef.current
        if (container) {
          const button = container.querySelector(`[data-recent-index="${newIndex}"]`) as HTMLElement
          button?.focus()
        }
      }
    },
    [recentEmojis, handleSelect, recentColumnCount]
  )

  // Keyboard navigation for main emoji grid
  const handleMainGridKeyDown = useCallback(
    (e: React.KeyboardEvent, currentIndex: number) => {
      if (filteredEmojis.length === 0) return

      let newIndex = currentIndex
      let handled = false

      switch (e.key) {
        case 'ArrowRight':
          if (currentIndex < filteredEmojis.length - 1) {
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
          if (nextRowIndex < filteredEmojis.length) {
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
            // Go to start of current row
            const currentRow = Math.floor(currentIndex / columnCount)
            newIndex = currentRow * columnCount
          }
          handled = true
          break
        case 'End':
          if (e.ctrlKey) {
            newIndex = filteredEmojis.length - 1
          } else {
            // Go to end of current row
            const currentRow = Math.floor(currentIndex / columnCount)
            newIndex = Math.min((currentRow + 1) * columnCount - 1, filteredEmojis.length - 1)
          }
          handled = true
          break
        case 'PageDown':
          newIndex = Math.min(currentIndex + columnCount * 3, filteredEmojis.length - 1)
          handled = true
          break
        case 'PageUp':
          newIndex = Math.max(currentIndex - columnCount * 3, 0)
          handled = true
          break
        case 'Enter':
        case ' ':
          e.preventDefault()
          if (filteredEmojis[currentIndex]) {
            handleSelect(filteredEmojis[currentIndex])
          }
          return
      }

      if (handled) {
        e.preventDefault()
        e.stopPropagation()
        setMainFocusedIndex(newIndex)

        // Scroll the grid to show the focused item
        if (gridRef.current) {
          const targetRow = Math.floor(newIndex / columnCount)
          const targetCol = newIndex % columnCount
          gridRef.current.scrollToCell({
            rowIndex: targetRow,
            columnIndex: targetCol,
            rowAlign: 'smart',
            columnAlign: 'smart',
          })
        }

        // Focus the new element after a small delay to allow scroll
        setTimeout(() => {
          const container = mainGridContainerRef.current
          if (container) {
            const button = container.querySelector(`[data-main-index="${newIndex}"]`) as HTMLElement
            button?.focus()
          }
        }, 10)
      }
    },
    [filteredEmojis, columnCount, handleSelect, gridRef]
  )

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="w-6 h-6 border-2 border-win11-bg-accent border-t-transparent rounded-full animate-spin" />
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Search bar */}
      <div className="px-3 pt-3 pb-2 flex-shrink-0">
        <SearchBar
          value={searchQuery}
          onChange={setSearchQuery}
          placeholder="Search emojis..."
          aria-label="Search emojis"
          isDark={isDark}
          opacity={opacity}
        />
      </div>

      {/* Recent emojis (only show when not searching) */}
      {!searchQuery && recentEmojis.length > 0 && (
        <div className="px-3 pb-2 flex-shrink-0">
          <div className="flex items-center gap-1.5 mb-1.5">
            <Clock className="w-3 h-3 dark:text-win11-text-tertiary text-win11Light-text-secondary" />
            <span className="text-xs dark:text-win11-text-tertiary text-win11Light-text-secondary">
              Recently used
            </span>
          </div>
          <div
            ref={recentGridRef}
            className="flex flex-wrap gap-1"
            role="grid"
            aria-label="Recently used emojis"
          >
            {recentEmojis.slice(0, 16).map((emoji, index) => (
              <button
                key={`recent-${emoji.char}`}
                data-recent-index={index}
                tabIndex={index === recentFocusedIndex ? 0 : -1}
                onClick={() => handleSelect(emoji)}
                onFocus={() => setRecentFocusedIndex(index)}
                onKeyDown={(e) => handleRecentKeyDown(e, index)}
                className={clsx(
                  'w-8 h-8 flex items-center justify-center text-xl',
                  'rounded-md transition-all duration-100',
                  'hover:bg-win11Light-bg-tertiary dark:hover:bg-win11-bg-card-hover',
                  'hover:scale-110',
                  'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent'
                )}
                title={emoji.name}
                aria-label={emoji.name}
              >
                {emoji.char}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Category pills (only show when not searching) */}
      {!searchQuery && (
        <div className="px-3 pb-2 flex-shrink-0 flex items-center gap-1">
          <button
            onClick={() => scrollCategories('left')}
            className="p-1 rounded-full hover:bg-win11Light-bg-tertiary dark:hover:bg-win11-bg-card-hover text-win11Light-text-secondary dark:text-win11-text-secondary"
            tabIndex={-1}
          >
            <ChevronLeft className="w-4 h-4" />
          </button>

          <div
            ref={scrollContainerRef}
            className="flex gap-1.5 overflow-x-hidden scroll-smooth flex-1"
            role="tablist"
            aria-label="Emoji categories"
          >
            <CategoryPill
              category="All"
              isActive={selectedCategory === null}
              onClick={() => setSelectedCategory(null)}
              tabIndex={categoryFocusedIndex === 0 ? 0 : -1}
              onKeyDown={(e) => handleCategoryKeyDown(e, 0)}
              onFocus={() => setCategoryFocusedIndex(0)}
              data-category-index={0}
              isDark={isDark}
              opacity={opacity}
            />
            {categories.map((cat, index) => (
              <CategoryPill
                key={cat}
                category={cat}
                isActive={selectedCategory === cat}
                onClick={() => setSelectedCategory(cat)}
                tabIndex={categoryFocusedIndex === index + 1 ? 0 : -1}
                onKeyDown={(e) => handleCategoryKeyDown(e, index + 1)}
                onFocus={() => setCategoryFocusedIndex(index + 1)}
                data-category-index={index + 1}
                isDark={isDark}
                opacity={opacity}
              />
            ))}
          </div>

          <button
            onClick={() => scrollCategories('right')}
            className="p-1 rounded-full hover:bg-win11Light-bg-tertiary dark:hover:bg-win11-bg-card-hover text-win11Light-text-secondary dark:text-win11-text-secondary"
            tabIndex={-1}
          >
            <ChevronRight className="w-4 h-4" />
          </button>
        </div>
      )}

      {/* Emoji grid */}
      <div className="flex-1 min-h-0 overflow-hidden" ref={containerRef}>
        {filteredEmojis.length === 0 && (
          <div className="flex flex-col items-center justify-center h-full py-8">
            <p className="text-sm dark:text-win11-text-secondary text-win11Light-text-secondary">
              No emojis found
            </p>
          </div>
        )}
        {filteredEmojis.length > 0 && dimensions.width > 0 && (
          <div
            ref={mainGridContainerRef}
            role="grid"
            aria-label="Emoji grid"
            style={{ height: gridHeight }}
          >
            <Grid<EmojiGridData>
              gridRef={gridRef}
              columnCount={columnCount}
              columnWidth={columnWidth}
              rowCount={rowCount}
              rowHeight={CELL_SIZE}
              defaultHeight={gridHeight}
              defaultWidth={gridWidth}
              className="scrollbar-win11"
              style={{ overflowX: 'hidden', overflowY: 'scroll' }}
              cellProps={{
                emojis: filteredEmojis,
                onSelect: handleSelect,
                onHover: setHoveredEmoji,
                focusedIndex: mainFocusedIndex,
                onKeyDown: handleMainGridKeyDown,
                onItemFocus: setMainFocusedIndex,
                columnCount,
                columnWidth,
              }}
              cellComponent={EmojiGridCell}
            />
          </div>
        )}
      </div>

      {/* Footer with hovered emoji info */}
      <div
        className={clsx(
          'px-3 py-2 h-10 flex-shrink-0',
          'border-t dark:border-win11-border-subtle border-win11Light-border',
          'flex items-center gap-2'
        )}
      >
        {hoveredEmoji ? (
          <>
            <span className="text-xl">{hoveredEmoji.char}</span>
            <span className="text-xs dark:text-win11-text-secondary text-win11Light-text-secondary truncate">
              {hoveredEmoji.name}
            </span>
          </>
        ) : (
          <span className="text-xs dark:text-win11-text-tertiary text-win11Light-text-secondary">
            Click to paste emoji
          </span>
        )}
      </div>
    </div>
  )
}
