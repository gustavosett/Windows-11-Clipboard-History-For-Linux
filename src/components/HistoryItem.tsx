import { useCallback, forwardRef } from 'react'
import { clsx } from 'clsx'
import { Pin, X, Image as ImageIcon, Type } from 'lucide-react'
import type { ClipboardItem } from '../types/clipboard'
import { getCardBackgroundStyle, getTertiaryBackgroundStyle } from '../utils/themeUtils'

interface HistoryItemProps {
  item: ClipboardItem
  onPaste: (id: string) => void
  onDelete: (id: string) => void
  onTogglePin: (id: string) => void
  onFocus?: () => void
  index: number
  isFocused?: boolean
  isDark: boolean
  secondaryOpacity: number
}

export const HistoryItem = forwardRef<HTMLDivElement, HistoryItemProps>(function HistoryItem(
  {
    item,
    onPaste,
    onDelete,
    onTogglePin,
    onFocus,
    index,
    isFocused = false,
    isDark,
    secondaryOpacity,
  },
  ref
) {
  const isText = item.content.type === 'Text' || item.content.type === 'RichText'

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
        isDark
          ? 'hover:bg-win11-bg-card-hover border border-win11-border-subtle'
          : 'hover:bg-win11Light-bg-card-hover border border-win11Light-border',
        // Pinned indicator
        item.pinned && 'ring-1 ring-win11-bg-accent',
        // Focus styles
        'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent'
      )}
      onClick={handleClick}
      onFocus={onFocus}
      role="button"
      tabIndex={isFocused ? 0 : -1}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault()
          handleClick()
        }
      }}
      style={{
        animationDelay: `${index * 30}ms`,
        ...getCardBackgroundStyle(isDark, secondaryOpacity),
      }}
    >
      {/* Content type indicator */}
      <div className="flex items-start gap-3">
        {/* Icon */}
        <div
          className={clsx('flex-shrink-0 w-8 h-8 rounded-md flex items-center justify-center')}
          style={getTertiaryBackgroundStyle(isDark, secondaryOpacity)}
        >
          {isText ? (
            <Type
              className={clsx(
                'w-4 h-4',
                isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
              )}
            />
          ) : (
            <ImageIcon
              className={clsx(
                'w-4 h-4',
                isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
              )}
            />
          )}
        </div>

        {/* Content */}
        <div className="flex-1 min-w-0">
          {item.content.type === 'Text' && (
            <p
              className={clsx(
                'text-sm line-clamp-3 break-words whitespace-pre-wrap',
                isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
              )}
            >
              {item.content.data}
            </p>
          )}

          {item.content.type === 'RichText' && (
            <p
              className={clsx(
                'text-sm line-clamp-3 break-words whitespace-pre-wrap',
                isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
              )}
            >
              {item.content.data.plain}
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
          <span
            className={clsx(
              'text-xs mt-1 block',
              isDark ? 'text-win11-text-tertiary' : 'text-win11Light-text-secondary'
            )}
          >
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
              isDark ? 'hover:bg-win11-bg-tertiary' : 'hover:bg-win11Light-bg-tertiary',
              item.pinned
                ? 'text-win11-bg-accent'
                : isDark
                  ? 'text-win11-text-tertiary'
                  : 'text-win11Light-text-secondary'
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
              isDark
                ? 'text-win11-text-tertiary hover:bg-win11-bg-tertiary'
                : 'text-win11Light-text-secondary hover:bg-win11Light-bg-tertiary',
              'hover:text-win11-error'
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
        <div className="absolute -top-1 -right-1 w-2 h-2 rounded-full bg-win11-bg-accent" />
      )}
    </div>
  )
})
