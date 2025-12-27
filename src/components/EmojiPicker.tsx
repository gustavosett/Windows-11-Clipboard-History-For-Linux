import { useState, useCallback, memo, useRef } from 'react'
import { Grid, useGridRef } from 'react-window'
import { clsx } from 'clsx'
import { Clock } from 'lucide-react'
import { useEmojiPicker } from '../hooks/useEmojiPicker'
import { SearchBar } from './common/SearchBar'
import type { Emoji } from '../services/emojiService'

import { PickerLayout } from './common/PickerLayout'
import { CategoryStrip } from './common/CategoryStrip'
import { useResponsiveGrid } from '../hooks/useResponsiveGrid'
import { useKeyboardNavigation } from '../hooks/useKeyboardNavigation'

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
  'data-recent-index'?: number
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
  'data-recent-index': recentIndex,
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
      data-recent-index={recentIndex}
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

  const { containerRef, dimensions } = useResponsiveGrid()

  const gridRef = useGridRef(null)
  const recentGridRef = useRef<HTMLDivElement>(null)
  const mainGridContainerRef = useRef<HTMLDivElement>(null)

  // Roving tabindex states
  const [recentFocusedIndex, setRecentFocusedIndex] = useState(0)
  const [mainFocusedIndex, setMainFocusedIndex] = useState(0)
  const [categoryFocusedIndex, setCategoryFocusedIndex] = useState(0)

  const handleSearchChange = useCallback(
    (val: string) => {
      setSearchQuery(val)
      setRecentFocusedIndex(0)
      setMainFocusedIndex(0)
    },
    [setSearchQuery]
  )

  const handleCategorySelect = useCallback(
    (cat: string | null) => {
      setSelectedCategory(cat)
      setRecentFocusedIndex(0)
      setMainFocusedIndex(0)
    },
    [setSelectedCategory]
  )

  // Handle emoji selection
  const handleSelect = useCallback(
    (emoji: Emoji) => {
      pasteEmoji(emoji)
    },
    [pasteEmoji]
  )

  // Calculate grid dimensions based on container
  const innerWidth = Math.max(0, dimensions.width - GRID_PADDING * 2)
  const columnCount = Math.max(1, Math.floor(innerWidth / CELL_SIZE))
  const columnWidth = columnCount > 0 ? innerWidth / columnCount : CELL_SIZE
  const rowCount = Math.ceil(filteredEmojis.length / columnCount)
  const recentColumnCount = 8

  const handleMainGridKeyDown = useKeyboardNavigation({
    items: filteredEmojis,
    columnCount,
    onSelect: handleSelect,
    setFocusedIndex: setMainFocusedIndex,
    gridRef,
    containerRef: mainGridContainerRef,
    dataAttributeName: 'data-main-index',
  })

  const handleRecentKeyDown = useKeyboardNavigation({
    items: recentEmojis.slice(0, 16),
    columnCount: recentColumnCount,
    onSelect: handleSelect,
    setFocusedIndex: setRecentFocusedIndex,
    containerRef: recentGridRef,
    dataAttributeName: 'data-recent-index',
  })

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="w-6 h-6 border-2 border-win11-bg-accent border-t-transparent rounded-full animate-spin" />
      </div>
    )
  }

  return (
    <PickerLayout
      header={
        <SearchBar
          value={searchQuery}
          onChange={handleSearchChange}
          placeholder="Search emojis..."
          aria-label="Search emojis"
          isDark={isDark}
          opacity={opacity}
        />
      }
      subHeader={
        <>
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
                  <div key={`recent-${emoji.char}`} className="w-8 h-8">
                    <EmojiCell
                      emoji={emoji}
                      onSelect={handleSelect}
                      onHover={setHoveredEmoji}
                      tabIndex={index === recentFocusedIndex ? 0 : -1}
                      data-recent-index={index}
                      onKeyDown={(e) => handleRecentKeyDown(e, index)}
                      onItemFocus={() => setRecentFocusedIndex(index)}
                    />
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Category pills */}
          {!searchQuery && (
            <CategoryStrip
              categories={categories}
              selectedCategory={selectedCategory}
              onSelectCategory={handleCategorySelect}
              focusedIndex={categoryFocusedIndex}
              setFocusedIndex={setCategoryFocusedIndex}
              isDark={isDark}
              opacity={opacity}
            />
          )}
        </>
      }
      footer={
        hoveredEmoji ? (
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
        )
      }
    >
      {/* Emoji grid */}
      <div className="h-full w-full" ref={containerRef}>
        {filteredEmojis.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full py-8">
            <p className="text-sm dark:text-win11-text-secondary text-win11Light-text-secondary">
              No emojis found
            </p>
          </div>
        ) : (
          dimensions.width > 0 &&
          dimensions.height > 0 && (
            <div
              ref={mainGridContainerRef}
              role="grid"
              aria-label="Emoji grid"
              style={{ height: dimensions.height }}
            >
              <Grid<EmojiGridData>
                gridRef={gridRef}
                columnCount={columnCount}
                columnWidth={columnWidth}
                rowCount={rowCount}
                rowHeight={CELL_SIZE}
                defaultHeight={dimensions.height}
                defaultWidth={dimensions.width}
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
          )
        )}
      </div>
    </PickerLayout>
  )
}
