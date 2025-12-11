/**
 * Emoji Picker Component
 * Windows 11 style emoji picker with virtualized grid for performance
 */
import { useState, useCallback, memo, useRef, useLayoutEffect } from 'react'
import { FixedSizeGrid as Grid } from 'react-window'
import { clsx } from 'clsx'
import { Search, Clock, X, ChevronLeft, ChevronRight } from 'lucide-react'
import { useEmojiPicker } from '../hooks/useEmojiPicker'
import type { Emoji } from '../services/emojiService'

/** Size of each emoji cell */
const CELL_SIZE = 40
/** Padding inside the grid container */
const GRID_PADDING = 12

interface EmojiCellProps {
  emoji: Emoji
  onSelect: (emoji: Emoji) => void
  onHover?: (emoji: Emoji | null) => void
}

/** Individual emoji cell - memoized for performance */
const EmojiCell = memo(function EmojiCell({ emoji, onSelect, onHover }: EmojiCellProps) {
  return (
    <button
      onClick={() => onSelect(emoji)}
      onMouseEnter={() => onHover?.(emoji)}
      onMouseLeave={() => onHover?.(null)}
      className={clsx(
        'flex items-center justify-center',
        'w-full h-full text-2xl',
        'rounded-md transition-all duration-100',
        'hover:bg-win11-bg-tertiary dark:hover:bg-win11-bg-card-hover',
        'hover:scale-110',
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
}

const CategoryPill = memo(function CategoryPill({
  category,
  isActive,
  onClick,
}: CategoryPillProps) {
  return (
    <button
      onClick={onClick}
      className={clsx(
        'px-3 py-1 text-xs rounded-full whitespace-nowrap',
        'transition-colors duration-150',
        'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent',
        isActive
          ? 'bg-win11-bg-accent text-white'
          : [
              'dark:bg-win11-bg-tertiary bg-win11Light-bg-tertiary',
              'dark:text-win11-text-secondary text-win11Light-text-secondary',
              'hover:dark:bg-win11-bg-card-hover hover:bg-win11Light-bg-card-hover',
            ]
      )}
    >
      {category}
    </button>
  )
})

export function EmojiPicker() {
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
  const [dimensions, setDimensions] = useState({ width: 0, height: 0 })

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

  // Cell renderer for virtualized grid
  const Cell = useCallback(
    (props: { columnIndex: number; rowIndex: number; style: React.CSSProperties }) => {
      const { columnIndex, rowIndex, style } = props
      const index = rowIndex * columnCount + columnIndex
      if (index >= filteredEmojis.length) {
        return null
      }

      const emoji = filteredEmojis[index]
      return (
        <div
          style={{
            ...style,
            left: Number(style.left) + GRID_PADDING,
            width: columnWidth,
            height: CELL_SIZE,
            padding: 4,
          }}
        >
          <EmojiCell emoji={emoji} onSelect={handleSelect} onHover={setHoveredEmoji} />
        </div>
      )
    },
    [filteredEmojis, handleSelect, columnCount, columnWidth]
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
        <div
          className={clsx(
            'flex items-center gap-2 px-3 py-2',
            'rounded-md',
            'dark:bg-win11-bg-tertiary bg-win11Light-bg-tertiary',
            'border dark:border-win11-border-subtle border-win11Light-border',
            'focus-within:ring-2 focus-within:ring-win11-bg-accent'
          )}
        >
          <Search className="w-4 h-4 dark:text-win11-text-tertiary text-win11Light-text-secondary flex-shrink-0" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search emojis..."
            className={clsx(
              'flex-1 bg-transparent border-none outline-none',
              'text-sm',
              'dark:text-win11-text-primary text-win11Light-text-primary',
              'placeholder:dark:text-win11-text-tertiary placeholder:text-win11Light-text-secondary'
            )}
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery('')}
              className="p-0.5 rounded dark:hover:bg-win11-bg-card-hover hover:bg-win11Light-bg-card-hover"
            >
              <X className="w-4 h-4 dark:text-win11-text-tertiary text-win11Light-text-secondary" />
            </button>
          )}
        </div>
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
          <div className="flex flex-wrap gap-1">
            {recentEmojis.slice(0, 16).map((emoji) => (
              <button
                key={`recent-${emoji.char}`}
                onClick={() => handleSelect(emoji)}
                className={clsx(
                  'w-8 h-8 flex items-center justify-center text-xl',
                  'rounded-md transition-all duration-100',
                  'hover:bg-win11-bg-tertiary dark:hover:bg-win11-bg-card-hover',
                  'hover:scale-110',
                  'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent'
                )}
                title={emoji.name}
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
            className="p-1 rounded-full hover:bg-win11-bg-tertiary dark:hover:bg-win11-bg-card-hover text-win11Light-text-secondary dark:text-win11-text-secondary"
          >
            <ChevronLeft className="w-4 h-4" />
          </button>

          <div
            ref={scrollContainerRef}
            className="flex gap-1.5 overflow-x-hidden scroll-smooth flex-1"
          >
            <CategoryPill
              category="All"
              isActive={selectedCategory === null}
              onClick={() => setSelectedCategory(null)}
            />
            {categories.map((cat) => (
              <CategoryPill
                key={cat}
                category={cat}
                isActive={selectedCategory === cat}
                onClick={() => setSelectedCategory(cat)}
              />
            ))}
          </div>

          <button
            onClick={() => scrollCategories('right')}
            className="p-1 rounded-full hover:bg-win11-bg-tertiary dark:hover:bg-win11-bg-card-hover text-win11Light-text-secondary dark:text-win11-text-secondary"
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
          <Grid
            columnCount={columnCount}
            columnWidth={columnWidth}
            height={gridHeight}
            rowCount={rowCount}
            rowHeight={CELL_SIZE}
            width={gridWidth}
            className="scrollbar-win11"
            style={{ overflowX: 'hidden' }}
          >
            {Cell}
          </Grid>
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
