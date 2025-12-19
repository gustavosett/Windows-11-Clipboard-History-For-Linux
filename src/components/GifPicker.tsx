/**
 * GIF Picker Component
 * Windows 11 style GIF picker with virtualized grid for performance
 */
import { useState, memo, useRef, useLayoutEffect, useCallback } from 'react'
import { Grid } from 'react-window'
import { clsx } from 'clsx'
import { Search, X, RefreshCw, TrendingUp } from 'lucide-react'
import { useGifPicker } from '../hooks/useGifPicker'
import type { Gif } from '../types/gif'

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
}

/** Individual GIF cell - memoized for performance */
const GifCell = memo(function GifCell({ gif, onSelect, style }: GifCellProps) {
  const [isLoaded, setIsLoaded] = useState(false)
  const [hasError, setHasError] = useState(false)

  return (
    <div style={style} className="p-1">
      <button
        onClick={() => onSelect(gif)}
        className={clsx(
          'w-full h-full rounded-lg overflow-hidden',
          'transition-all duration-150',
          'hover:ring-2 hover:ring-win11-bg-accent hover:scale-[1.02]',
          'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent',
          'relative',
          'dark:bg-win11-bg-tertiary bg-win11Light-bg-tertiary'
        )}
        title={gif.title}
        aria-label={gif.title}
      >
        {/* Loading skeleton */}
        {!isLoaded && !hasError && (
          <div className="absolute inset-0 animate-pulse dark:bg-win11-bg-tertiary bg-win11Light-bg-tertiary" tabIndex={-1} />
        )}

        {/* Error state */}
        {hasError && (
          <div className="absolute inset-0 flex items-center justify-center text-xs dark:text-win11-text-disabled text-win11Light-text-disabled" tabIndex={-1}>
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

export function GifPicker() {
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

  const containerRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLInputElement>(null)
  const [dimensions, setDimensions] = useState({ width: 0, height: 0 })

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
    inputRef.current?.focus()
  }, [setSearchQuery])

  // Calculate grid dimensions
  const gridWidth = dimensions.width - GRID_PADDING * 2
  const columnWidth = gridWidth / COLUMN_COUNT
  const rowCount = Math.ceil(gifs.length / COLUMN_COUNT)
  const gridHeight = dimensions.height

  // Grid cell renderer
  const CellRenderer = useCallback(
    ({
      columnIndex,
      rowIndex,
      style,
    }: {
      columnIndex: number
      rowIndex: number
      style: React.CSSProperties
    }) => {
      const index = rowIndex * COLUMN_COUNT + columnIndex
      const gif = gifs[index]

      if (!gif) {
        return <></>
      }

      return <GifCell gif={gif} onSelect={handleSelect} style={style} />
    },
    [gifs, handleSelect]
  )

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
        <Grid<object>
          columnCount={COLUMN_COUNT}
          columnWidth={columnWidth}
          rowCount={rowCount}
          rowHeight={CELL_HEIGHT}
          className="scrollbar-thin scrollbar-thumb-win11-border-subtle scrollbar-track-transparent"
          style={{ height: gridHeight, width: gridWidth }}
          cellComponent={CellRenderer}
          cellProps={{}}
        />
      )
    }

    return null
  }

  return (
    <div className="flex flex-col h-full overflow-hidden relative">
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

      {/* Search Bar */}
      <div className="px-3 pt-3 pb-2 flex-shrink-0">
        <div className="relative">
          <Search
            size={16}
            className="absolute left-3 top-1/2 -translate-y-1/2 dark:text-win11-text-disabled text-win11Light-text-disabled"
          />
          <input
            ref={inputRef}
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search Tenor GIFs..."
            className={clsx(
              'w-full h-9 pl-9 pr-16 rounded-lg',
              'text-sm',
              'dark:bg-win11-bg-tertiary bg-win11Light-bg-tertiary',
              'dark:text-win11-text-primary text-win11Light-text-primary',
              'placeholder:dark:text-win11-text-disabled placeholder:text-win11Light-text-disabled',
              'border dark:border-win11-border-subtle border-win11Light-border-subtle',
              'focus:outline-none focus:ring-2 focus:ring-win11-bg-accent',
              'transition-all duration-150'
            )}
          />
          <div className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1">
            {searchQuery && (
              <button
                onClick={handleClearSearch}
                className={clsx(
                  'p-1 rounded',
                  'dark:text-win11-text-disabled text-win11Light-text-disabled',
                  'hover:dark:text-win11-text-primary hover:text-win11Light-text-primary',
                  'hover:dark:bg-win11-bg-card-hover hover:bg-win11Light-bg-card-hover',
                  'transition-colors duration-150'
                )}
                title="Clear search"
              >
                <X size={14} />
              </button>
            )}
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
            >
              <TrendingUp size={14} />
            </button>
          </div>
        </div>
      </div>

      {/* Category indicator */}
      <div className="px-3 pb-2 flex-shrink-0">
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
      </div>

      {/* GIF Grid Container */}
      <div ref={containerRef} className="flex-1 min-h-0 overflow-hidden px-3 pb-3">
        {renderGridContent()}
      </div>

      {/* Tenor Attribution */}
      <div className="px-3 py-2 text-center flex-shrink-0">
        <span className="text-[10px] dark:text-win11-text-disabled text-win11Light-text-disabled">
          Powered by Tenor
        </span>
      </div>
    </div>
  )
}
