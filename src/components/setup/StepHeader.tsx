import { clsx } from 'clsx'
import type { LucideIcon } from 'lucide-react'

interface StepHeaderProps {
  readonly icon: LucideIcon
  readonly title: string
  readonly subtitle?: React.ReactNode
  readonly isDark: boolean
  /** Welcome/done use a larger 64px medallion; inner steps use 56px. */
  /** خوش‌آمد/پایان مدال ۶۴ پیکسلی دارند؛ گام‌های میانی ۵۶. */
  readonly large?: boolean
}

/**
 * The circular icon medallion + title + subtitle shared by every wizard
 * step. One definition keeps vertical rhythm identical across steps.
 * / مدال دایره‌ای + عنوان + زیرعنوان مشترک همهٔ گام‌ها تا ریتم عمودی
 * یکسان بماند.
 */
export function StepHeader({ icon: Icon, title, subtitle, isDark, large = false }: StepHeaderProps) {
  return (
    <div className={clsx('text-center mb-6')}>
      <div
        className={clsx(
          'mx-auto rounded-full flex items-center justify-center',
          large ? 'w-16 h-16 mb-2' : 'w-14 h-14 mb-4',
          isDark ? 'bg-win11-bg-tertiary' : 'bg-win11Light-bg-tertiary'
        )}
      >
        <Icon
          className={clsx(
            large ? 'w-8 h-8' : 'w-7 h-7',
            isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
          )}
        />
      </div>
      <h2
        className={clsx(
          large ? 'text-xl mb-2' : 'text-lg mb-1',
          'font-semibold',
          isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
        )}
      >
        {title}
      </h2>
      {subtitle && (
        <p
          className={clsx(
            'text-sm',
            isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
          )}
        >
          {subtitle}
        </p>
      )}
    </div>
  )
}
