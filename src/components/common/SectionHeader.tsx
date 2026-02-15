import { type ReactNode } from 'react'
import { clsx } from 'clsx'

export interface SectionHeaderProps {
  /** Icon to display (e.g., <TrendingUp size={12} /> or <Clock size={12} />) */
  icon?: ReactNode
  /** Label content to display (supports text, styled elements, or localization components) */
  label: ReactNode
  /** Additional content to display on the right (e.g., loading spinner) */
  rightContent?: ReactNode
  /** Additional CSS classes */
  className?: string
}

/**
 * Shared component for section headers like "Trending GIFs" and "Recently used"
 */
export function SectionHeader({ icon, label, rightContent, className }: SectionHeaderProps) {
  return (
    <div
      className={clsx(
        'flex items-center gap-1.5 text-xs',
        'dark:text-win11-text-tertiary text-win11Light-text-tertiary',
        className
      )}
    >
      {icon}
      <span>{label}</span>
      {rightContent && <div className="ml-auto">{rightContent}</div>}
    </div>
  )
}
