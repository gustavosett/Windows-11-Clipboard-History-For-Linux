import { ClipboardList } from 'lucide-react'
import { clsx } from 'clsx'
import { useTranslation } from 'react-i18next'

interface EmptyStateProps {
  isDark: boolean
}

/**
 * Empty state shown before the first clipboard capture.
 * / حالت خالی قبل از اولین ثبت کلیپ‌بورد.
 */
export function EmptyState({ isDark }: EmptyStateProps) {
  const { t } = useTranslation()
  const shortcut = t('clipboard.empty_shortcut')
  const parts = shortcut.split(/\s*\+\s*/)

  return (
    <div
      className="flex flex-col items-center justify-center h-full py-10 px-5 text-center select-none"
      data-tauri-drag-region
      role="status"
      aria-live="polite"
    >
      <div
        className={clsx(
          'relative w-[5.5rem] h-[5.5rem] rounded-[1.35rem] flex items-center justify-center mb-6',
          'shadow-[0_8px_24px_rgba(0,0,0,0.12)] empty-icon-glow',
          isDark
            ? 'bg-gradient-to-br from-white/12 to-white/4 ring-1 ring-white/12'
            : 'bg-gradient-to-br from-white to-slate-100 ring-1 ring-black/5'
        )}
      >
        <div
          className={clsx(
            'absolute inset-2 rounded-[1.1rem] opacity-70',
            isDark ? 'bg-win11-bg-accent/20' : 'bg-win11-bg-accent/12'
          )}
          aria-hidden
        />
        <ClipboardList
          className={clsx(
            'relative w-9 h-9',
            isDark ? 'text-sky-300' : 'text-win11-bg-accent'
          )}
          aria-hidden
        />
      </div>

      <h3
        className={clsx(
          'text-base font-semibold mb-1.5 tracking-tight',
          isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
        )}
      >
        {t('clipboard.empty_state')}
      </h3>

      <p
        className={clsx(
          'text-sm max-w-[250px] leading-relaxed',
          isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
        )}
      >
        {t('clipboard.empty_state_desc')}
      </p>

      <div className="mt-5 flex items-center gap-1.5" aria-label={shortcut}>
        {parts.map((part, i) => (
          <span key={`${part}-${i}`} className="flex items-center gap-1.5">
            {i > 0 && (
              <span
                className={clsx(
                  'text-xs font-medium',
                  isDark ? 'text-win11-text-tertiary' : 'text-win11Light-text-secondary'
                )}
              >
                +
              </span>
            )}
            <kbd
              className={clsx(
                'min-w-[1.75rem] px-2 py-1 rounded-md text-[11px] font-semibold tracking-wide',
                'shadow-sm',
                isDark
                  ? 'bg-white/8 text-win11-text-primary ring-1 ring-white/12'
                  : 'bg-white text-win11Light-text-primary ring-1 ring-black/8'
              )}
            >
              {part}
            </kbd>
          </span>
        ))}
      </div>

      <p
        className={clsx(
          'mt-3 text-[11px] max-w-[240px] leading-relaxed',
          isDark ? 'text-win11-text-tertiary' : 'text-win11Light-text-secondary'
        )}
      >
        {t('clipboard.empty_hint')}
      </p>
    </div>
  )
}
