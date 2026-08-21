import { clsx } from 'clsx'
import type { ReactNode } from 'react'

interface SectionCardProps {
  /** Optional leading icon rendered inside a tinted tile. */
  icon?: ReactNode
  title: string
  subtitle?: string
  isDark: boolean
  /** Renders the card at reduced opacity (e.g. when the feature is unavailable). */
  dimmed?: boolean
  /** When false, the header is not separated by a bottom border. */
  dividedHeader?: boolean
  children: ReactNode
}

/**
 * Shared visual container for every Settings section.
 * Keeps the look of the settings window consistent (uniform radius, border,
 * header layout) and removes duplicated markup from section components.
 */
export function SectionCard({
  icon,
  title,
  subtitle,
  isDark,
  dimmed = false,
  dividedHeader = true,
  children,
}: SectionCardProps) {
  return (
    <section
      className={clsx(
        'rounded-xl border shadow-sm transition-all',
        isDark ? 'bg-win11-bg-secondary border-white/5' : 'bg-white border-gray-200/60',
        dimmed && 'opacity-60'
      )}
    >
      <div className={clsx('p-6', dividedHeader && 'border-b border-inherit')}>
        <div className="flex items-center gap-3">
          {icon && (
            <div className={clsx('p-2 rounded-lg', isDark ? 'bg-white/5' : 'bg-gray-100')}>
              {icon}
            </div>
          )}
          <div>
            <h2 className="text-base font-semibold">{title}</h2>
            {subtitle && (
              <p className={clsx('text-xs mt-0.5', isDark ? 'text-gray-400' : 'text-gray-500')}>
                {subtitle}
              </p>
            )}
          </div>
        </div>
      </div>
      <div className="p-6">{children}</div>
    </section>
  )
}
