import { useCallback, forwardRef, useMemo, useState, useRef, useEffect } from 'react'
import { clsx } from 'clsx'
import { Pin, X, Image as ImageIcon, Type, Wrench, ExternalLink, Mail } from 'lucide-react'
import type { ClipboardItem } from '../types/clipboard'
import { getCardBackgroundStyle, getTertiaryBackgroundStyle } from '../utils/themeUtils'
import { transformerService } from '../services/transformerService'
import { smartActionService } from '../services/smartActionService'
import type { SmartAction } from '../services/smartActionService'
import type { TransformerActionType } from '../services/transformerService'

interface HistoryItemProps {
  item: ClipboardItem
  onPaste: (id: string) => void
  onDelete: (id: string) => void
  onTogglePin: (id: string) => void
  onFocus?: () => void
  onTransform?: (originalItem: ClipboardItem, newContent: string) => void
  index: number
  isFocused?: boolean
  isDark: boolean
  secondaryOpacity: number
  isCompact?: boolean
  // Feature flags passed from parent
  enableSmartActions: boolean
  enableDevTools: boolean
  enableUiPolish: boolean
}

export const HistoryItem = forwardRef<HTMLDivElement, HistoryItemProps>(function HistoryItem(
  {
    item,
    onPaste,
    onDelete,
    onTogglePin,
    onFocus,
    onTransform,
    index,
    isFocused = false,
    isDark,
    secondaryOpacity,
    isCompact = false,
    enableSmartActions,
    enableDevTools,
    enableUiPolish,
  },
  ref
) {
  const isText = item.content.type === 'Text'
  const [showTools, setShowTools] = useState(false)
  const [toolsMenuPos, setToolsMenuPos] = useState({ top: 0, left: 0 })
  const toolsButtonRef = useRef<HTMLButtonElement>(null)
  
  // Use compact mode only if enabled by flag
  const effectiveCompact = enableUiPolish ? isCompact : false

  // Detect available transformer actions (memoized)
  const actions = useMemo(() => {
    if (!enableDevTools) return []
    if (item.content.type === 'Text') {
      return transformerService.detectActions(item.content.data)
    }
    return []
  }, [item.content, enableDevTools])

  // Detect smart actions (memoized)
  const smartActions = useMemo(() => {
    if (!enableSmartActions) return []
    if (item.content.type === 'Text') {
        return smartActionService.detectActions(item.content.data)
    }
    return []
  }, [item.content, enableSmartActions])

  const colorPreview = smartActions.find(a => a.id === 'color-preview')
  const linkAction = smartActions.find(a => a.id === 'open-link')
  const emailAction = smartActions.find(a => a.id === 'compose-email')

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
  
  // Handle smart action execution
  const handleSmartAction = useCallback(async (e: React.MouseEvent, action: SmartAction) => {
      e.stopPropagation()
      await smartActionService.execute(action)
  }, [])

  // Toggle tools menu
  const handleToggleTools = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation()
      if (showTools) {
        setShowTools(false)
      } else {
        // Calculate position relative to viewport due to overflow issues if nested
        const rect = toolsButtonRef.current?.getBoundingClientRect()
        if (rect) {
            // Position slightly below and to the left of the button
            setToolsMenuPos({ top: rect.bottom + 5, left: rect.left - 100 })
        }
        setShowTools(true)
      }
    },
    [showTools]
  )
  
  // Close menu on click outside
  useEffect(() => {
      if (!showTools) return
      
      const listener = () => setShowTools(false)
      window.addEventListener('click', listener)
      return () => window.removeEventListener('click', listener)
  }, [showTools])

  // Handle transformation
  const handleAction = useCallback((e: React.MouseEvent, actionId: string) => {
    e.stopPropagation()
    if (item.content.type === 'Text') {
        const newContent = transformerService.transform(item.content.data, actionId as TransformerActionType)
        onTransform?.(item, newContent)
    }
    setShowTools(false)
  }, [item, onTransform])

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
          className={clsx(
              'flex-shrink-0 rounded-md flex items-center justify-center',
              effectiveCompact ? 'w-6 h-6' : 'w-8 h-8',
              // If color preview, use the color as background
             colorPreview && 'shadow-sm'
          )}
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
                effectiveCompact ? 'w-3 h-3' : 'w-4 h-4',
                isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
              )}
            />
          ) : (
            <ImageIcon
              className={clsx(
                effectiveCompact ? 'w-3 h-3' : 'w-4 h-4',
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
                'text-sm break-words whitespace-pre-wrap',
                 effectiveCompact ? 'line-clamp-1' : 'line-clamp-3',
                isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
              )}
            >
              {item.content.data}
            </p>
          )}

          {item.content.type === 'Image' && (
            <div className="relative">
              {!effectiveCompact ? (
                  <>
                      <img
                        src={`data:image/png;base64,${item.content.data.base64}`}
                        alt="Clipboard image"
                        className="max-w-full max-h-24 rounded object-contain bg-black/10"
                      />
                      <span className="absolute bottom-1 right-1 text-xs px-1.5 py-0.5 rounded bg-black/60 text-white">
                        {item.content.data.width}×{item.content.data.height}
                      </span>
                  </>
              ) : (
                  <span className={clsx("text-sm italic", isDark ? 'text-win11-text-tertiary' : 'text-win11Light-text-secondary')}>
                      Image ({item.content.data.width}×{item.content.data.height})
                  </span>
              )}
            </div>
          )}

          {/* Timestamp */}
          {!effectiveCompact && (
              <span
                className={clsx(
                  'text-xs mt-1 block',
                  isDark ? 'text-win11-text-tertiary' : 'text-win11Light-text-secondary'
                )}
              >
                {formatTime(item.timestamp)}
              </span>
          )}
        </div>

        {/* Action buttons - visible on hover */}
        <div
          className={clsx(
            'flex items-center gap-1 opacity-0 group-hover:opacity-100',
            'transition-opacity duration-150',
            // Also visible if tools menu is open for THIS item
            showTools && 'opacity-100'
          )}
        >
           {/* Smart Actions Buttons */}
           {linkAction && (
               <button
                 onClick={(e) => handleSmartAction(e, linkAction)}
                 className={clsx(
                   'p-1.5 rounded-md transition-colors',
                   isDark
                     ? 'text-win11-text-tertiary hover:bg-win11-bg-tertiary'
                     : 'text-win11Light-text-secondary hover:bg-win11Light-bg-tertiary'
                 )}
                 title="Open Link"
                 tabIndex={-1}
               >
                 <ExternalLink className="w-4 h-4" />
               </button>
           )}
           
           {emailAction && (
               <button
                 onClick={(e) => handleSmartAction(e, emailAction)}
                 className={clsx(
                   'p-1.5 rounded-md transition-colors',
                   isDark
                     ? 'text-win11-text-tertiary hover:bg-win11-bg-tertiary'
                     : 'text-win11Light-text-secondary hover:bg-win11Light-bg-tertiary'
                 )}
                 title="Compose Email"
                 tabIndex={-1}
               >
                 <Mail className="w-4 h-4" />
               </button>
           )}

          {/* Tools button (Only if actions available) */}
          {actions.length > 0 && (
              <button
                ref={toolsButtonRef}
                onClick={handleToggleTools}
                className={clsx(
                  'p-1.5 rounded-md transition-colors',
                  isDark
                    ? 'text-win11-text-tertiary hover:bg-win11-bg-tertiary'
                    : 'text-win11Light-text-secondary hover:bg-win11Light-bg-tertiary',
                   showTools && 'bg-win11-bg-tertiary text-win11-text-primary'
                )}
                title="Tools"
                tabIndex={-1}
              >
                  <Wrench className="w-4 h-4" />
              </button>
          )}

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

      {/* Tools Menu (Portal or Fixed position) */}
      {showTools && (
        <div 
          className={clsx(
              "fixed z-50 py-1 rounded-md shadow-lg border backdrop-blur-md min-w-[150px]",
               isDark 
                ? "bg-win11-bg-card/90 border-win11-border-subtle text-win11-text-primary" 
                : "bg-win11Light-bg-card/90 border-win11Light-border text-win11Light-text-primary"
          )}
          style={{
              top: toolsMenuPos.top,
              left: Math.max(10, Math.min(window.innerWidth - 170, toolsMenuPos.left)), // Keep within bounds
          }}
          onClick={(e) => e.stopPropagation()} // Prevent click through to item
        >
            <div className={clsx("px-3 py-1.5 text-xs font-semibold opacity-70", isDark ? "text-win11-text-tertiary" : "text-win11Light-text-secondary")}>
                Tools
            </div>
            {actions.map(action => (
                <button
                    key={action.id}
                    onClick={(e) => handleAction(e, action.id)}
                    className={clsx(
                        "w-full text-left px-3 py-1.5 text-sm transition-colors",
                         isDark 
                          ? "hover:bg-win11-bg-card-hover" 
                          : "hover:bg-win11Light-bg-card-hover"
                    )}
                >
                    {action.label}
                </button>
            ))}
        </div>
      )}
    </div>
  )
})
