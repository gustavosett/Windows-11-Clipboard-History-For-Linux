import { forwardRef, memo, type ReactNode } from 'react'
import { clsx } from 'clsx'
import { Search, X } from 'lucide-react'
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
            'w-full h-9 pl-9 pr-16 rounded-lg',
            'text-sm',
            'dark:text-win11-text-primary text-win11Light-text-primary',
            'placeholder:dark:text-win11-text-disabled placeholder:text-win11Light-text-disabled',
            'focus:outline-none focus:ring-2 focus:ring-win11-bg-accent',
            'transition-all duration-150'
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
          {rightActions}
        </div>
      </div>
    )
  })
)

export default SearchBar
