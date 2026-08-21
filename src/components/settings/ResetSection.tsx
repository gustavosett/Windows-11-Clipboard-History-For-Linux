import { clsx } from 'clsx'
import { useTranslation } from 'react-i18next'
import { ResetIcon } from './icons'

interface ResetSectionProps {
  isDark: boolean
  onReset: () => void
}

/** Danger-zone: restore every setting to its default and restart the wizard. */
export function ResetSection({ isDark, onReset }: ResetSectionProps) {
  const { t } = useTranslation()

  return (
    <div className="flex justify-end pt-2">
      <button
        onClick={onReset}
        className={clsx(
          'flex items-center gap-2 px-4 py-2 text-sm font-medium rounded-lg transition-all',
          'hover:bg-red-50 hover:text-red-600',
          isDark ? 'text-gray-400 hover:bg-red-500/10 hover:text-red-400' : 'text-gray-500'
        )}
      >
        <ResetIcon />
        {t('settings_page.reset_defaults')}
      </button>
    </div>
  )
}
