import { clsx } from 'clsx'

export type StatusType = 'success' | 'warning' | 'error'

interface StatusCardProps {
  readonly type: StatusType
  readonly isDark: boolean
  readonly children: React.ReactNode
  readonly className?: string
}

/**
 * Coloured feedback card (success / warning / error) with theme-aware
 * light and dark variants. Replaces the old `statusCardClass` factory so
 * every step shares one visual definition.
 * / کارت بازخورد رنگی با نسخهٔ روشن و تاریک؛ جایگزین کارخانهٔ
 * `statusCardClass` تا همهٔ گام‌ها یک تعریف بصری داشته باشند.
 */
export function StatusCard({ type, isDark, children, className }: StatusCardProps) {
  return (
    <div
      className={clsx(
        'p-4 rounded-win11 flex items-start gap-3 text-sm',
        type === 'success' &&
          (isDark
            ? 'bg-win11-success/15 text-win11-success border border-win11-success/20'
            : 'bg-green-50 text-green-700 border border-green-200'),
        type === 'warning' &&
          (isDark
            ? 'bg-win11-warning/15 text-win11-warning border border-win11-warning/20'
            : 'bg-amber-50 text-amber-700 border border-amber-200'),
        type === 'error' &&
          (isDark
            ? 'bg-win11-error/15 text-win11-error border border-win11-error/20'
            : 'bg-red-50 text-red-700 border border-red-200'),
        className
      )}
    >
      {children}
    </div>
  )
}
