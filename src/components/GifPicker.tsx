import { useState, memo, useRef, useCallback } from 'react'
import { Grid, useGridRef } from 'react-window'
import { clsx } from 'clsx'
import { Search, RefreshCw, TrendingUp } from 'lucide-react'
import { useGifPicker } from '../hooks/useGifPicker'
import { SearchBar } from './common/SearchBar'
import type { Gif } from '../types/gif'
import { getTertiaryBackgroundStyle } from '../utils/themeUtils'

import { PickerLayout } from './common/PickerLayout'
import { useResponsiveGrid } from '../hooks/useResponsiveGrid'
import { useKeyboardNavigation } from '../hooks/useKeyboardNavigation'

/** Number of columns in the grid */
const COLUMN_COUNT = 2
/** Height of each GIF cell */
const CELL_HEIGHT = 120
/** Padding inside the grid container */
const GRID_PADDING = 12

interface GifCellProps {
  gif: Gif
  onSelect: (gif: Gif) => void
  style: React.CSSProperties
  tabIndex?: number
  'data-gif-index'?: number
  onKeyDown?: (e: React.KeyboardEvent) => void
  onItemFocus?: () => void
}

/** Individual GIF cell - memoized for performance */
const GifCell = memo(function GifCell({
  gif,
  onSelect,
  style,
  tabIndex = -1,
  'data-gif-index': gifIndex,
  onKeyDown,
  onItemFocus,
}: GifCellProps) {
  const [isLoaded, setIsLoaded] = useState(false)
  const [hasError, setHasError] = useState(false)

  return (
    <div style={style} className="p-1">
      <button
        onClick={() => onSelect(gif)}
        onFocus={onItemFocus}
        onKeyDown={onKeyDown}
        tabIndex={tabIndex}
        data-gif-index={gifIndex}
        className={clsx(
          'w-full h-full rounded-lg overflow-hidden',
          'transition-transform duration-150',
          'hover:ring-2 hover:ring-win11-bg-accent hover:scale-[1.02]',
          'transform-gpu will-change-transform',
          'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent',
          'relative',
          'dark:bg-win11-bg-tertiary bg-win11Light-bg-tertiary'
        )}
        title={gif.title}
        aria-label={gif.title}
      >
        {/* Loading skeleton */}
        {!isLoaded && !hasError && (
          <div className="absolute inset-0 animate-pulse dark:bg-win11-bg-tertiary bg-win11Light-bg-tertiary" />
        )}

        {/* Error state */}
        {hasError && (
          <div className="absolute inset-0 flex items-center justify-center text-xs dark:text-win11-text-disabled text-win11Light-text-disabled">
            Failed
          </div>
        )}

        {/* GIF Image */}
        <img
          src={gif.previewUrl}
          alt={gif.title}
          loading="lazy"
          className={clsx(
            'w-full h-full object-cover',
            isLoaded ? 'opacity-100' : 'opacity-0',
            'transition-opacity duration-200'
          )}
          onLoad={() => setIsLoaded(true)}
          onError={() => setHasError(true)}
        />
      </button>
    </div>
  )
})

/** Skeleton keys for loading state */
const SKELETON_KEYS = ['sk-1', 'sk-2', 'sk-3', 'sk-4', 'sk-5', 'sk-6', 'sk-7', 'sk-8'] as const

/** Loading skeleton grid */
function LoadingSkeleton() {
  return (
    <div className="grid grid-cols-2 gap-2 p-3">
      {SKELETON_KEYS.map((key) => (
        <div
          key={key}
          className={clsx(
            'h-[112px] rounded-lg animate-pulse',
            'dark:bg-win11-bg-tertiary bg-win11Light-bg-tertiary'
          )}
        />
      ))}
    </div>
  )
}

/** Empty/Error state props */
interface EmptyStateProps {
  readonly message: string
  readonly isError?: boolean
}

/** Empty/Error state */
function EmptyState({ message, isError }: EmptyStateProps) {
  return (
    <div className="flex flex-col items-center justify-center h-full py-12 px-4 text-center">
      <p
        className={clsx(
          'text-sm',
          isError
            ? 'text-red-500 dark:text-red-400'
            : 'dark:text-win11-text-secondary text-win11Light-text-secondary'
        )}
      >
        {message}
      </p>
    </div>
  )
}

interface GifGridData {
  gifs: Gif[]
  onSelect: (gif: Gif) => void
  focusedIndex: number
  onKeyDown: (e: React.KeyboardEvent, index: number) => void
  onItemFocus: (index: number) => void
}

function GifGridCell({
  columnIndex,
  rowIndex,
  style,
  gifs,
  onSelect,
  focusedIndex,
  onKeyDown,
  onItemFocus,
  ariaAttributes,
}: {
  columnIndex: number
  rowIndex: number
  style: React.CSSProperties
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ariaAttributes: any
} & GifGridData) {
  const index = rowIndex * COLUMN_COUNT + columnIndex

  if (index >= gifs.length) {
    return <></>
  }

  const gif = gifs[index]
  const isFocused = index === focusedIndex

  return (
    <div {...ariaAttributes} style={style} className="p-1">
      <GifCell
        gif={gif}
        onSelect={onSelect}
        style={{ width: '100%', height: '100%' }}
        tabIndex={isFocused ? 0 : -1}
        data-gif-index={index}
        onKeyDown={(e) => onKeyDown(e, index)}
        onItemFocus={() => onItemFocus(index)}
      />
    </div>
  )
}

export interface GifPickerProps {
  isDark: boolean
  opacity: number
}

export function GifPicker({ isDark, opacity }: GifPickerProps) {
  const {
    searchQuery,
    setSearchQuery,
    gifs,
    isLoading,
    isPasting,
    error,
    pasteGif,
    refreshTrending,
  } = useGifPicker()

  const inputRef = useRef<HTMLInputElement>(null)
  const gridRef = useGridRef(null)
  const gridContainerRef = useRef<HTMLDivElement>(null)
  const [focusedIndex, setFocusedIndex] = useState(0)

  // Use shared hook for sizing
  const { containerRef, dimensions } = useResponsiveGrid()

  // Handle GIF selection
  const handleSelect = useCallback(
    (gif: Gif) => {
      pasteGif(gif)
    },
    [pasteGif]
  )

  // Clear search
  const handleClearSearch = useCallback(() => {
    setSearchQuery('')
    setFocusedIndex(0)
    inputRef.current?.focus()
  }, [setSearchQuery])

  // Calculate grid dimensions
  const gridWidth = dimensions.width - GRID_PADDING * 2
  const columnWidth = gridWidth / COLUMN_COUNT
  const rowCount = Math.ceil(gifs.length / COLUMN_COUNT)
  const gridHeight = dimensions.height

  // Use shared hook for keyboard navigation
  const handleGridKeyDown = useKeyboardNavigation({
    items: gifs,
    columnCount: COLUMN_COUNT,
    onSelect: handleSelect,
    setFocusedIndex,
    gridRef,
    containerRef: gridContainerRef,
    dataAttributeName: 'data-gif-index',
  })

  // Render grid content based on state
  const renderGridContent = () => {
    if (isLoading && gifs.length === 0) {
      return <LoadingSkeleton />
    }

    if (error) {
      return <EmptyState message={error} isError />
    }

    if (gifs.length === 0) {
      return <EmptyState message="No GIFs found. Try a different search!" />
    }

    if (dimensions.width > 0 && dimensions.height > 0) {
      return (
        <div
          ref={gridContainerRef}
          role="grid"
          aria-label="GIF grid"
          style={{ height: gridHeight }}
        >
          <Grid<GifGridData>
            gridRef={gridRef}
            columnCount={COLUMN_COUNT}
            columnWidth={columnWidth}
            rowCount={rowCount}
            rowHeight={CELL_HEIGHT}
            defaultHeight={gridHeight}
            defaultWidth={gridWidth}
            className="scrollbar-win11"
            style={{ overflowY: 'scroll' }}
            cellProps={{
              gifs,
              onSelect: handleSelect,
              focusedIndex,
              onKeyDown: handleGridKeyDown,
              onItemFocus: setFocusedIndex,
            }}
            cellComponent={GifGridCell}
          />
        </div>
      )
    }

    return null
  }

  return (
    <PickerLayout
      header={
        <SearchBar
          ref={inputRef}
          value={searchQuery}
          onChange={(val: string) => setSearchQuery(val)}
          onClear={handleClearSearch}
          placeholder="Search Tenor GIFs..."
          aria-label="Search Tenor GIFs"
          isDark={isDark}
          opacity={opacity}
          rightActions={
            <button
              onClick={refreshTrending}
              className={clsx(
                'p-1 rounded',
                'dark:text-win11-text-disabled text-win11Light-text-disabled',
                'hover:dark:text-win11-text-primary hover:text-win11Light-text-primary',
                'hover:dark:bg-win11-bg-card-hover hover:bg-win11Light-bg-card-hover',
                'transition-colors duration-150'
              )}
              title="Show trending"
              style={getTertiaryBackgroundStyle(isDark, opacity)}
            >
              <TrendingUp size={14} />
            </button>
          }
        />
      }
      subHeader={
        <div className="flex items-center gap-2 text-xs dark:text-win11-text-secondary text-win11Light-text-secondary">
          {searchQuery ? (
            <>
              <Search size={12} />
              <span>Results for "{searchQuery}"</span>
            </>
          ) : (
            <>
              <TrendingUp size={12} />
              <span>Trending GIFs</span>
            </>
          )}
          {isLoading && <RefreshCw size={12} className="animate-spin ml-auto" />}
        </div>
      }
      footer={
        <div className="w-full text-center">
          <span className="text-[10px] dark:text-win11-text-disabled text-win11Light-text-disabled">
            Powered by Tenor
          </span>
        </div>
      }
    >
      {/* Loading Overlay */}
      {isPasting && (
        <div className="absolute inset-0 z-50 flex flex-col items-center justify-center bg-black/20 backdrop-blur-[1px]">
          <div className="bg-win11Light-bg-card dark:bg-win11-bg-card p-4 rounded-xl shadow-lg flex flex-col items-center gap-3 border border-win11Light-border-subtle dark:border-win11-border-subtle">
            <div className="w-8 h-8 border-4 border-win11-bg-accent border-t-transparent rounded-full animate-spin" />
            <span className="text-sm font-medium text-win11Light-text-primary dark:text-win11-text-primary">
              Pasting GIF...
            </span>
          </div>
        </div>
      )}

      {/* Grid Container */}
      <div ref={containerRef} className="h-full w-full px-3 pb-3">
        {renderGridContent()}
      </div>
    </PickerLayout>
  )
}
