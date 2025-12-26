import { useCallback, forwardRef } from 'react'
import { clsx } from 'clsx'
import { Pin, X, Image as ImageIcon, Type } from 'lucide-react'
import type { ClipboardItem } from '../../types/clipboard'
import { getCardBackgroundStyle, getTertiaryBackgroundStyle } from '../../utils/themeUtils'
import { useSmartActions } from '../../hooks/useSmartActions'
import { HistorySmartActions } from '../HistorySmartActions'

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
  isCompact?: boolean
  // Feature flags passed from parent
  enableSmartActions: boolean
  enableUiPolish: boolean
}

import {
  TextContent,
  ImageContent,
  Timestamp,
} from './_HistoryItemContent'
import {
  getIconSize,
  getIconContainerClasses,
} from './_HistoryItemUtils'

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
    isCompact = false,
    enableSmartActions,
    enableUiPolish,
  },
  ref
) {
  const isText = item.content.type === 'Text' || item.content.type === 'RichText'

  // Use compact mode only if enabled by flag
  const effectiveCompact = enableUiPolish ? isCompact : false
  const iconSize = getIconSize(effectiveCompact)
  const iconContainerClasses = getIconContainerClasses(effectiveCompact)

  // Smart Actions Hook
  const { colorPreview, linkAction, emailAction, handleSmartAction } = useSmartActions(
    item,
    enableSmartActions
  )

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
        'group relative rounded-win11 cursor-pointer',
        effectiveCompact ? 'p-2' : 'p-3',
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
          className={clsx(iconContainerClasses, colorPreview && 'shadow-sm')}
          style={
            colorPreview && colorPreview.data
              ? { backgroundColor: colorPreview.data }
              : getTertiaryBackgroundStyle(isDark, secondaryOpacity)
          }
          title={colorPreview ? `Color: ${colorPreview.data}` : undefined}
        >
          {colorPreview ? null : isText ? (
            <Type
              className={clsx(
                iconSize,
                isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
              )}
            />
          ) : (
            <ImageIcon
              className={clsx(
                iconSize,
                isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
              )}
            />
          )}
        </div>

        {/* Content */}
        <div className="flex-1 min-w-0">
          <TextContent item={item} isDark={isDark} effectiveCompact={effectiveCompact} />
          <ImageContent item={item} isDark={isDark} effectiveCompact={effectiveCompact} />
          <Timestamp show={!effectiveCompact} isDark={isDark} timestamp={item.timestamp} />
        </div>

        {/* Action buttons - visible on hover */}
        <div
          className={clsx(
            'flex items-center gap-1 opacity-0 group-hover:opacity-100',
            'transition-opacity duration-150'
          )}
        >
          {/* Smart Actions Buttons */}
          <HistorySmartActions
            linkAction={linkAction}
            emailAction={emailAction}
            isDark={isDark}
            onActionClick={handleSmartAction}
          />

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
