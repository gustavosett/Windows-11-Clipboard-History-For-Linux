/**
 * Symbol Picker Component
 * Windows 11 style symbol picker with virtualized grid for performance
 */
import { useState, useCallback, memo, useRef, useLayoutEffect } from 'react'
import { Grid, useGridRef } from 'react-window'
import { clsx } from 'clsx'
import { Clock, ChevronLeft, ChevronRight } from 'lucide-react'
import { useSymbolPicker } from '../hooks/useSymbolPicker'
import { SearchBar } from './SearchBar'
import { getTertiaryBackgroundStyle } from '../utils/themeUtils'
import type { SymbolItem } from '../services/symbolService'

/** Size of each symbol cell */
const CELL_SIZE = 40
/** Padding inside the grid container */
const GRID_PADDING = 12

interface SymbolCellProps {
  symbol: SymbolItem
  onSelect: (symbol: SymbolItem) => void
  onHover?: (symbol: SymbolItem | null) => void
  tabIndex?: number
  'data-main-index'?: number
  onKeyDown?: (e: React.KeyboardEvent) => void
  onItemFocus?: () => void
}

/** Individual symbol cell - memoized for performance */
const SymbolCell = memo(function SymbolCell({
  symbol,
  onSelect,
  onHover,
  tabIndex = -1,
  'data-main-index': mainIndex,
  onKeyDown,
  onItemFocus,
}: SymbolCellProps) {
  return (
    <button
      onClick={() => onSelect(symbol)}
      onMouseEnter={() => onHover?.(symbol)}
      onMouseLeave={() => onHover?.(null)}
      onFocus={onItemFocus}
      onKeyDown={onKeyDown}
      tabIndex={tabIndex}
      data-main-index={mainIndex}
      className={clsx(
        'flex items-center justify-center',
        'w-full h-full text-xl',
        'rounded-md transition-transform duration-100',
        'hover:bg-win11Light-bg-tertiary dark:hover:bg-win11-bg-card-hover',
        'hover:scale-110 transform-gpu hover:will-change-transform',
        'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent'
      )}
      title={symbol.name}
    >
      {symbol.char}
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

interface SymbolGridData {
  symbols: SymbolItem[]
  onSelect: (symbol: SymbolItem) => void
  onHover: (symbol: SymbolItem | null) => void
  focusedIndex: number
  onKeyDown: (e: React.KeyboardEvent, index: number) => void
  onItemFocus: (index: number) => void
  columnCount: number
  columnWidth: number
}

function SymbolGridCell({
  columnIndex,
  rowIndex,
  style,
  symbols,
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
  ariaAttributes: React.AriaAttributes
} & SymbolGridData) {
  const index = rowIndex * columnCount + columnIndex
  if (index >= symbols.length) {
    return <></>
  }

  const symbol = symbols[index]
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
      <SymbolCell
        symbol={symbol}
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

export interface SymbolPickerProps {
  isDark: boolean
  opacity: number
}

export function SymbolPicker({ isDark, opacity }: SymbolPickerProps) {
  const {
    searchQuery,
    setSearchQuery,
    selectedCategory,
    setSelectedCategory,
    categories,
    filteredSymbols,
    recentSymbols,
    pasteSymbol,
  } = useSymbolPicker()

  const [hoveredSymbol, setHoveredSymbol] = useState<SymbolItem | null>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const scrollContainerRef = useRef<HTMLDivElement>(null)
  const gridRef = useGridRef(null)
  const recentGridRef = useRef<HTMLDivElement>(null)
  const [dimensions, setDimensions] = useState({ width: 0, height: 0 })

  // Roving tabindex state for recent symbols
  const [recentFocusedIndex, setRecentFocusedIndex] = useState(0)
  // Roving tabindex state for main symbol grid
  const [mainFocusedIndex, setMainFocusedIndex] = useState(0)
  // Roving tabindex state for categories
  const [categoryFocusedIndex, setCategoryFocusedIndex] = useState(0)

  const handleSearchChange = useCallback((query: string) => {
    setSearchQuery(query)
    setRecentFocusedIndex(0)
    setMainFocusedIndex(0)
  }, [setSearchQuery])

  const handleCategorySelect = useCallback((category: string | null) => {
    setSelectedCategory(category)
    setRecentFocusedIndex(0)
    setMainFocusedIndex(0)
  }, [setSelectedCategory])

  // Measure container size
  useLayoutEffect(() => {
    const updateSize = () => {
      if (containerRef.current) {
        const { width, height } = containerRef.current.getBoundingClientRect()
        if (width > 0 && height > 0) {
          setDimensions((prev) => {
            if (prev.width !== width || prev.height !== height) {
              return { width, height }
            }
            return prev
          })
        }
      }
    }

    updateSize()
    const rafId = requestAnimationFrame(updateSize)
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

  // Handle symbol selection
  const handleSelect = useCallback(
    (symbol: SymbolItem) => {
      pasteSymbol(symbol)
    },
    [pasteSymbol]
  )

  // Calculate grid dimensions
  const innerWidth = Math.max(0, dimensions.width - GRID_PADDING * 2)
  const columnCount = Math.max(1, Math.floor(innerWidth / CELL_SIZE))
  const columnWidth = columnCount > 0 ? innerWidth / columnCount : CELL_SIZE
  const rowCount = Math.ceil(filteredSymbols.length / columnCount)
  const gridHeight = dimensions.height > 0 ? dimensions.height : 200
  const recentColumnCount = 10 // Slightly more dense for symbols than emojis if needed, but 8-10 is safe

  // Keyboard navigation for categories
  const handleCategoryKeyDown = useCallback(
    (e: React.KeyboardEvent, currentIndex: number) => {
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
            handleCategorySelect(null)
          } else {
            handleCategorySelect(categories[currentIndex - 1])
          }
          return
      }

      if (handled) {
        e.preventDefault()
        e.stopPropagation()
        setCategoryFocusedIndex(newIndex)
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
    [categories, handleCategorySelect]
  )

  // Keyboard navigation for recent symbols grid
  const handleRecentKeyDown = useCallback(
    (e: React.KeyboardEvent, currentIndex: number) => {
      const recentCount = Math.min(recentSymbols.length, 16)
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
          if (recentSymbols[currentIndex]) {
            handleSelect(recentSymbols[currentIndex])
          }
          return
      }

      if (handled) {
        e.preventDefault()
        e.stopPropagation()
        setRecentFocusedIndex(newIndex)
        const container = recentGridRef.current
        if (container) {
          const button = container.querySelector(`[data-recent-index="${newIndex}"]`) as HTMLElement
          button?.focus()
        }
      }
    },
    [recentSymbols, handleSelect, recentColumnCount]
  )

  // Keyboard navigation for main symbol grid
  const handleMainGridKeyDown = useCallback(
    (e: React.KeyboardEvent, currentIndex: number) => {
      if (filteredSymbols.length === 0) return

      let newIndex = currentIndex
      let handled = false

      switch (e.key) {
        case 'ArrowRight':
          if (currentIndex < filteredSymbols.length - 1) {
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
          if (nextRowIndex < filteredSymbols.length) {
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
            newIndex = filteredSymbols.length - 1
          } else {
            const currentRow = Math.floor(currentIndex / columnCount)
            newIndex = Math.min((currentRow + 1) * columnCount - 1, filteredSymbols.length - 1)
          }
          handled = true
          break
        case 'PageDown':
          newIndex = Math.min(currentIndex + columnCount * 3, filteredSymbols.length - 1)
          handled = true
          break
        case 'PageUp':
          newIndex = Math.max(currentIndex - columnCount * 3, 0)
          handled = true
          break
        case 'Enter':
        case ' ':
          e.preventDefault()
          if (filteredSymbols[currentIndex]) {
            handleSelect(filteredSymbols[currentIndex])
          }
          return
      }

      if (handled) {
        e.preventDefault()
        e.stopPropagation()
        setMainFocusedIndex(newIndex)
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
      }
    },
    [filteredSymbols, handleSelect, columnCount, gridRef]
  )

  return (
    <div className="flex flex-col h-full overflow-hidden select-none">
      {/* Search bar */}
      <div className="px-3 pt-3 pb-2 flex-shrink-0">
        <SearchBar
          value={searchQuery}
          onChange={handleSearchChange}
          placeholder="Search symbols..."
          isDark={isDark}
          opacity={opacity}
        />
      </div>

      {/* Recent symbols (only show when not searching and on All category) */}
      {!searchQuery && !selectedCategory && recentSymbols.length > 0 && (
        <div className="px-3 pb-2 flex-shrink-0 border-b dark:border-win11-border-subtle border-win11Light-border mb-2">
          <div className="flex items-center gap-1.5 mb-1.5">
            <Clock className="w-3 h-3 dark:text-win11-text-tertiary text-win11Light-text-secondary" />
            <span className="text-xs dark:text-win11-text-tertiary text-win11Light-text-secondary">
              Recently used
            </span>
          </div>
          <div 
             ref={recentGridRef}
             className="flex flex-wrap gap-1 pb-2"
          >
            {recentSymbols.slice(0, 16).map((symbol, index) => (
              <div 
                key={`recent-${symbol.char}-${index}`} 
                className="w-10 h-10 p-0.5"
              >
                <SymbolCell
                  symbol={symbol}
                  onSelect={handleSelect}
                  onHover={setHoveredSymbol}
                  tabIndex={index === recentFocusedIndex ? 0 : -1}
                  data-main-index={index}
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
          >
            <CategoryPill
              category="All"
              isActive={selectedCategory === null}
              onClick={() => handleCategorySelect(null)}
              tabIndex={categoryFocusedIndex === 0 ? 0 : -1}
              onKeyDown={(e) => handleCategoryKeyDown(e, 0)}
              onFocus={() => setCategoryFocusedIndex(0)}
              data-category-index={0}
              isDark={isDark}
              opacity={opacity}
            />
            {categories.map((cat, idx) => (
              <CategoryPill
                key={cat}
                category={cat}
                isActive={selectedCategory === cat}
                onClick={() => handleCategorySelect(cat)}
                tabIndex={categoryFocusedIndex === idx + 1 ? 0 : -1}
                onKeyDown={(e) => handleCategoryKeyDown(e, idx + 1)}
                onFocus={() => setCategoryFocusedIndex(idx + 1)}
                data-category-index={idx + 1}
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

      {/* Symbol grid */}
      <div 
        ref={containerRef}
        className="flex-1 min-h-0 overflow-hidden border-t dark:border-win11-border-subtle border-win11Light-border"
      >
        {filteredSymbols.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full py-8">
            <p className="text-sm dark:text-win11-text-secondary text-win11Light-text-secondary">
              No symbols found
            </p>
          </div>
        ) : (
          <div 
             role="grid"
             aria-label="Symbol grid"
             style={{ height: gridHeight }}
          >
            {dimensions.width > 0 && dimensions.height > 0 && (
              <Grid<SymbolGridData>
                gridRef={gridRef}
                columnCount={columnCount}
                columnWidth={columnWidth}
                rowCount={rowCount}
                rowHeight={CELL_SIZE}
                defaultHeight={gridHeight}
                defaultWidth={dimensions.width}
                className="scrollbar-win11"
                style={{ overflowX: 'hidden', overflowY: 'scroll' }}
                cellProps={{
                  symbols: filteredSymbols,
                  onSelect: handleSelect,
                  onHover: setHoveredSymbol,
                  focusedIndex: mainFocusedIndex,
                  onKeyDown: handleMainGridKeyDown,
                  onItemFocus: setMainFocusedIndex,
                  columnCount,
                  columnWidth,
                }}
                cellComponent={SymbolGridCell}
              />
            )}
          </div>
        )}
      </div>

      {/* Footer with hovered symbol info */}
      <div
        className={clsx(
          'px-3 py-2 h-10 flex-shrink-0',
          'border-t dark:border-win11-border-subtle border-win11Light-border',
          'flex items-center gap-2'
        )}
      >
        {hoveredSymbol ? (
          <>
            <span className="text-xl">{hoveredSymbol.char}</span>
            <span className="text-xs dark:text-win11-text-secondary text-win11Light-text-secondary truncate">
              {hoveredSymbol.name}
            </span>
          </>
        ) : (
          <span className="text-xs dark:text-win11-text-tertiary text-win11Light-text-secondary">
            Click to paste symbol
          </span>
        )}
      </div>
    </div>
  )
}
