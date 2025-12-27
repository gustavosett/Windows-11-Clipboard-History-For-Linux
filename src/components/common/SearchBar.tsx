import { forwardRef, memo, type ReactNode } from 'react'
import { clsx } from 'clsx'
import { Search, X, Regex } from 'lucide-react'
import { getTertiaryBackgroundStyle } from '@/utils/themeUtils'

export interface SearchBarProps {
  value: string
  onChange: (value: string) => void
  placeholder?: string
  onClear?: () => void
  rightActions?: ReactNode
  'aria-label'?: string
  isDark: boolean
  opacity: number
  isRegex?: boolean
  onToggleRegex?: () => void
}

export const SearchBar = memo(
  forwardRef<HTMLInputElement, SearchBarProps>(function SearchBar(
    {
      value,
      onChange,
      placeholder = 'Search...',
      onClear,
      rightActions,
      'aria-label': ariaLabel,
      isDark,
      opacity,
      isRegex = false,
      onToggleRegex,
    },
    ref
  ) {
    const backgroundColor = getTertiaryBackgroundStyle(isDark, opacity).backgroundColor

    const handleClear = () => {
      onChange('')
      onClear?.()
    }

    return (
      <div className="relative">
        <Search
          size={16}
          className="absolute left-3 top-1/2 -translate-y-1/2 dark:text-win11-text-disabled text-win11Light-text-disabled pointer-events-none"
          aria-hidden="true"
        />
        <input
          ref={ref}
          type="text"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          aria-label={ariaLabel ?? placeholder}
          className={clsx(
            'w-full h-9 pl-9 rounded-lg',
            'text-sm',
            'dark:text-win11-text-primary text-win11Light-text-primary',
            'placeholder:dark:text-win11-text-disabled placeholder:text-win11Light-text-disabled',
            'focus:outline-none focus:ring-2 focus:ring-win11-bg-accent',
            'transition-all duration-150',
            // Adjust padding-right based on what buttons are present
            (() => {
              let padding = 'pr-8' // Default padding for clearing button
              if (onToggleRegex) padding = 'pr-16'
              if (rightActions) padding = onToggleRegex ? 'pr-24' : 'pr-16'
              return padding
            })()
          )}
          style={{ backgroundColor }}
        />
        <div className="absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1">
          {value && (
            <button
              type="button"
              onClick={handleClear}
              className={clsx(
                'p-1 rounded',
                'dark:text-win11-text-disabled text-win11Light-text-disabled',
                'hover:dark:text-win11-text-primary hover:text-win11Light-text-primary',
                'hover:dark:bg-win11-bg-card-hover hover:bg-win11Light-bg-card-hover',
                'transition-colors duration-150'
              )}
              title="Clear search"
              aria-label="Clear search"
            >
              <X size={14} />
            </button>
          )}

          {onToggleRegex && (
            <button
              type="button"
              onClick={onToggleRegex}
              className={clsx(
                'p-1 rounded',
                'transition-colors duration-150',
                isRegex
                  ? 'text-win11-bg-accent bg-win11-bg-accent/10'
                  : 'dark:text-win11-text-secondary text-win11Light-text-secondary hover:dark:bg-win11-bg-card-hover hover:bg-win11Light-bg-card-hover'
              )}
              title="Toggle Regex search"
              aria-label="Toggle Regex search"
            >
              <Regex size={14} />
            </button>
          )}

          {rightActions}
        </div>
      </div>
    )
  })
)

export default SearchBar
