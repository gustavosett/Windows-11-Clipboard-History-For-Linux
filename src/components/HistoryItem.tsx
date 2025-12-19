import { useCallback, forwardRef } from 'react'
import { clsx } from 'clsx'
import { Pin, X, Image as ImageIcon, Type } from 'lucide-react'
import type { ClipboardItem } from '../types/clipboard'

interface HistoryItemProps {
  item: ClipboardItem
  onPaste: (id: string) => void
  onDelete: (id: string) => void
  onTogglePin: (id: string) => void
  onFocus?: () => void
  index: number
}

export const HistoryItem = forwardRef<HTMLDivElement, HistoryItemProps>(function HistoryItem(
  { item, onPaste, onDelete, onTogglePin, onFocus, index },
  ref
) {
  const isText = item.content.type === 'Text'

  // Format timestamp
  const formatTime = useCallback((timestamp: string) => {
    const date = new Date(timestamp)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffMins = Math.floor(diffMs / 60000)
    const diffHours = Math.floor(diffMs / 3600000)

    if (diffMins < 1) return 'Just now'
    if (diffMins < 60) return `${diffMins}m ago`
    if (diffHours < 24) return `${diffHours}h ago`
    return date.toLocaleDateString()
  }, [])

  // Handle paste on click
  const handleClick = useCallback(() => {
    onPaste(item.id)
  }, [item.id, onPaste])

  // Handle delete with stopPropagation
  const handleDelete = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation()
      onDelete(item.id)
    },
    [item.id, onDelete]
  )

  // Handle pin toggle with stopPropagation
  const handleTogglePin = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation()
      onTogglePin(item.id)
    },
    [item.id, onTogglePin]
  )

  return (
    <div
      ref={ref}
      className={clsx(
        // Base styles
        'group relative rounded-win11 p-3 cursor-pointer',
        'transition-all duration-150 ease-out',
        // Animation delay based on index
        'animate-in',
        // Dark mode styles
        'dark:bg-win11-bg-card dark:hover:bg-win11-bg-card-hover',
        'dark:border dark:border-win11-border-subtle',
        // Light mode styles
        'bg-win11Light-bg-card hover:bg-win11Light-bg-card-hover',
        'border border-win11Light-border',
        // Pinned indicator
        item.pinned && 'ring-1 ring-win11-bg-accent',
        // Focus styles
        'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent'
      )}
      onClick={handleClick}
      onFocus={onFocus}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault()
          handleClick()
        }
      }}
      style={{ animationDelay: `${index * 30}ms` }}
    >
      {/* Content type indicator */}
      <div className="flex items-start gap-3">
        {/* Icon */}
        <div
          className={clsx(
            'flex-shrink-0 w-8 h-8 rounded-md flex items-center justify-center',
            'dark:bg-win11-bg-tertiary bg-win11Light-bg-tertiary'
          )}
        >
          {isText ? (
            <Type className="w-4 h-4 dark:text-win11-text-secondary text-win11Light-text-secondary" />
          ) : (
            <ImageIcon className="w-4 h-4 dark:text-win11-text-secondary text-win11Light-text-secondary" />
          )}
        </div>

        {/* Content */}
        <div className="flex-1 min-w-0">
          {item.content.type === 'Text' && (
            <p className="text-sm dark:text-win11-text-primary text-win11Light-text-primary line-clamp-3 break-words whitespace-pre-wrap">
              {item.content.data}
            </p>
          )}

          {item.content.type === 'Image' && (
            <div className="relative">
              <img
                src={`data:image/png;base64,${item.content.data.base64}`}
                alt="Clipboard image"
                className="max-w-full max-h-24 rounded object-contain bg-black/10"
              />
              <span className="absolute bottom-1 right-1 text-xs px-1.5 py-0.5 rounded bg-black/60 text-white">
                {item.content.data.width}×{item.content.data.height}
              </span>
            </div>
          )}

          {/* Timestamp */}
          <span className="text-xs dark:text-win11-text-tertiary text-win11Light-text-secondary mt-1 block">
            {formatTime(item.timestamp)}
          </span>
        </div>

        {/* Action buttons - visible on hover */}
        <div
          className={clsx(
            'flex items-center gap-1 opacity-0 group-hover:opacity-100',
            'transition-opacity duration-150'
          )}
        >
          {/* Pin button */}
          <button
            onClick={handleTogglePin}
            className={clsx(
              'p-1.5 rounded-md transition-colors',
              'hover:dark:bg-win11-bg-tertiary hover:bg-win11Light-bg-tertiary',
              item.pinned
                ? 'text-win11-bg-accent'
                : 'dark:text-win11-text-tertiary text-win11Light-text-secondary'
            )}
            title={item.pinned ? 'Unpin' : 'Pin'}
            tabIndex={-1}
          >
            <Pin className="w-4 h-4" fill={item.pinned ? 'currentColor' : 'none'} />
          </button>

          {/* Delete button */}
          <button
            onClick={handleDelete}
            className={clsx(
              'p-1.5 rounded-md transition-colors',
              'dark:text-win11-text-tertiary text-win11Light-text-secondary',
              'hover:text-win11-error hover:dark:bg-win11-bg-tertiary hover:bg-win11Light-bg-tertiary'
            )}
            title="Delete"
            tabIndex={-1}
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Pinned badge */}
      {item.pinned && (
        <div className="absolute -top-1 -right-1 w-2 h-2 rounded-full bg-win11-bg-accent" tabIndex={-1} />
      )}
    </div>
  )
})
