import { memo } from 'react'
import { clsx } from 'clsx'
import { getTertiaryBackgroundStyle } from '../utils/themeUtils'

export interface CategoryPillProps {
  category: string
  isActive: boolean
  onClick: () => void
  tabIndex?: number
  onKeyDown?: (e: React.KeyboardEvent) => void
  onFocus?: () => void
  'data-category-index'?: number
  isDark: boolean
  opacity: number
}

export const CategoryPill = memo(function CategoryPill({
  category,
  isActive,
  onClick,
  tabIndex = 0,
  onKeyDown,
  onFocus,
  'data-category-index': index,
  isDark,
  opacity,
}: CategoryPillProps) {
  return (
    <button
      onClick={onClick}
      tabIndex={tabIndex}
      onKeyDown={onKeyDown}
      onFocus={onFocus}
      data-category-index={index}
      className={clsx(
        'px-3 py-1 text-xs rounded-full whitespace-nowrap',
        'transition-colors duration-150',
        'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent',
        isActive
          ? 'bg-win11-bg-accent text-white'
          : [
              'text-win11Light-text-secondary dark:text-win11-text-secondary',
              'hover:dark:bg-win11-bg-card-hover hover:bg-win11Light-bg-card-hover',
            ]
      )}
      style={!isActive ? getTertiaryBackgroundStyle(isDark, opacity) : undefined}
    >
      {category}
    </button>
  )
})
