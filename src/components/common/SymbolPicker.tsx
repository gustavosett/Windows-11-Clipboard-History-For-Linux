import { useState, memo, useRef } from 'react'
import { Grid, useGridRef } from 'react-window'
import { clsx } from 'clsx'
import { Clock } from 'lucide-react'

// Componentes Refatorados
import { PickerLayout } from './PickerLayout'
import { CategoryStrip } from './CategoryStrip'
import { SymbolItem } from '@/services/symbolService'
import { useSymbolPicker } from '@/hooks/useSymbolPicker'
import { useResponsiveGrid } from '@/hooks/useResponsiveGrid'
import { useKeyboardNavigation } from '@/hooks/useKeyboardNavigation'
import SearchBar from '../SearchBar'

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

  const { containerRef, dimensions } = useResponsiveGrid()

  const gridRef = useGridRef(null)
  const recentGridRef = useRef<HTMLDivElement>(null)

  const [recentFocusedIndex, setRecentFocusedIndex] = useState(0)
  const [mainFocusedIndex, setMainFocusedIndex] = useState(0)
  const [categoryFocusedIndex, setCategoryFocusedIndex] = useState(0)

  const innerWidth = Math.max(0, dimensions.width - GRID_PADDING * 2)
  const columnCount = Math.max(1, Math.floor(innerWidth / CELL_SIZE))
  const columnWidth = columnCount > 0 ? innerWidth / columnCount : CELL_SIZE
  const rowCount = Math.ceil(filteredSymbols.length / columnCount)
  const recentColumnCount = 10

  const handleMainGridKeyDown = useKeyboardNavigation({
    items: filteredSymbols,
    columnCount,
    onSelect: pasteSymbol,
    setFocusedIndex: setMainFocusedIndex,
    gridRef,
  })

  const handleRecentKeyDown = useKeyboardNavigation({
    items: recentSymbols.slice(0, 16),
    columnCount: recentColumnCount,
    onSelect: pasteSymbol,
    setFocusedIndex: setRecentFocusedIndex,
    containerRef: recentGridRef,
    dataAttributeName: 'data-recent-index',
  })

  return (
    <PickerLayout
      header={
        <SearchBar
          value={searchQuery}
          onChange={(val) => {
            setSearchQuery(val)
            setRecentFocusedIndex(0)
            setMainFocusedIndex(0)
          }}
          placeholder="Search symbols..."
          isDark={isDark}
          opacity={opacity}
        />
      }
      subHeader={
        <>
          {/* Recent symbols */}
          {!searchQuery && !selectedCategory && recentSymbols.length > 0 && (
            <div className="border-b dark:border-win11-border-subtle border-win11Light-border mb-2">
              <div className="flex items-center gap-1.5 mb-1.5">
                <Clock className="w-3 h-3 dark:text-win11-text-tertiary text-win11Light-text-secondary" />
                <span className="text-xs dark:text-win11-text-tertiary text-win11Light-text-secondary">
                  Recently used
                </span>
              </div>
              <div ref={recentGridRef} className="flex flex-wrap gap-1 pb-2">
                {recentSymbols.slice(0, 16).map((symbol, index) => (
                  <div key={`recent-${symbol.char}-${index}`} className="w-10 h-10 p-0.5">
                    <SymbolCell
                      symbol={symbol}
                      onSelect={pasteSymbol}
                      onHover={setHoveredSymbol}
                      tabIndex={index === recentFocusedIndex ? 0 : -1}
                      data-main-index={index}
                      data-recent-index={index} // Necessário para o hook
                      onKeyDown={(e) => handleRecentKeyDown(e, index)}
                      onItemFocus={() => setRecentFocusedIndex(index)}
                    />
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Categories */}
          {!searchQuery && (
            <CategoryStrip
              categories={categories}
              selectedCategory={selectedCategory}
              onSelectCategory={(cat) => {
                setSelectedCategory(cat)
                setRecentFocusedIndex(0)
                setMainFocusedIndex(0)
              }}
              focusedIndex={categoryFocusedIndex}
              setFocusedIndex={setCategoryFocusedIndex}
              isDark={isDark}
              opacity={opacity}
            />
          )}
        </>
      }
      footer={
        hoveredSymbol ? (
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
        )
      }
    >
      {/* Main Grid */}
      <div
        ref={containerRef}
        className="h-full w-full border-t dark:border-win11-border-subtle border-win11Light-border"
      >
        {filteredSymbols.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full py-8">
            <p className="text-sm dark:text-win11-text-secondary text-win11Light-text-secondary">
              No symbols found
            </p>
          </div>
        ) : (
          dimensions.width > 0 &&
          dimensions.height > 0 && (
            <Grid<SymbolGridData>
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
                symbols: filteredSymbols,
                onSelect: pasteSymbol,
                onHover: setHoveredSymbol,
                focusedIndex: mainFocusedIndex,
                onKeyDown: handleMainGridKeyDown, // Hook
                onItemFocus: setMainFocusedIndex,
                columnCount,
                columnWidth,
              }}
              cellComponent={SymbolGridCell}
            />
          )
        )}
      </div>
    </PickerLayout>
  )
}
