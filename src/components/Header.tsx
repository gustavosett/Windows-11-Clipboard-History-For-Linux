import { clsx } from 'clsx'
import { useState } from 'react'
import { getTertiaryBackgroundStyle } from '../utils/themeUtils'

interface HeaderProps {
  onClearHistory: () => void
  itemCount: number
  isDark: boolean
  tertiaryOpacity: number
}

/**
 * Header component with title and action buttons
 */
export function Header({ onClearHistory, itemCount, isDark, tertiaryOpacity }: HeaderProps) {
  const [isHovered, setIsHovered] = useState(false)

  return (
    <div className="flex items-center justify-between px-4 py-3" data-tauri-drag-region>
      <div className="flex items-center gap-2">
        <h1
          className={clsx(
            'text-sm font-semibold',
            'select-none',
            isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
          )}
        >
          Clipboard
        </h1>
        {itemCount > 0 && (
          <span
            className={clsx(
              'text-xs px-2 py-0.5 rounded-full',
              'select-none',
              isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
            )}
            style={getTertiaryBackgroundStyle(isDark, tertiaryOpacity)}
          >
            {itemCount}
          </span>
        )}
      </div>

      <div className="flex items-center gap-1">
        {/* Clear history button */}
        <button
          onClick={onClearHistory}
          disabled={itemCount === 0}
          tabIndex={-1}
          onMouseEnter={() => setIsHovered(true)}
          onMouseLeave={() => setIsHovered(false)}
          className={clsx(
            'no-drag',
            'p-2 rounded-md transition-colors',
            'select-none',
            isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary',
            'disabled:opacity-50 disabled:cursor-not-allowed',
            'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent'
          )}
          style={
            isHovered && itemCount > 0
              ? getTertiaryBackgroundStyle(isDark, tertiaryOpacity)
              : undefined
          }
          title="Clear all"
        >
          <span className="text-xs">Clear All</span>
        </button>
      </div>
    </div>
  )
}
