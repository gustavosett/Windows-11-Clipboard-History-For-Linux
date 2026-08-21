import { clsx } from 'clsx'
import { useTranslation } from 'react-i18next'
import type { UserSettings } from '../../types/clipboard'
import { TrashIcon } from './icons'
import { SectionCard } from './SectionCard'

interface AutoDeleteSectionProps {
  settings: UserSettings
  isDark: boolean
  onIntervalChange: (value: string) => void
  onUnitChange: (unit: UserSettings['auto_delete_unit']) => void
}

const UNITS = ['minutes', 'hours', 'days', 'weeks'] as const

/** Automatic history cleanup configuration. */
export function AutoDeleteSection({
  settings,
  isDark,
  onIntervalChange,
  onUnitChange,
}: AutoDeleteSectionProps) {
  const { t } = useTranslation()

  return (
    <SectionCard
      icon={<TrashIcon />}
      title={t('settings_page.auto_delete_title')}
      subtitle={t('settings_page.auto_delete_subtitle')}
      dividedHeader={false}
      isDark={isDark}
    >
      <div className="flex flex-col sm:flex-row gap-4">
        <div className="flex-1 flex flex-col gap-2">
          <label className="text-xs font-medium opacity-60 ml-1" htmlFor="auto-delete-value">
            {t('settings_page.time_value')}
          </label>
          <input
            id="auto-delete-value"
            type="number"
            min="0"
            value={settings.auto_delete_interval || ''}
            placeholder={t('settings_page.disabled_placeholder')}
            onChange={(e) => onIntervalChange(e.target.value)}
            className={clsx(
              'w-full px-4 py-2.5 rounded-lg border outline-none transition-all font-medium',
              isDark
                ? 'bg-white/5 border-white/10 focus:border-win11-bg-accent text-white'
                : 'bg-gray-50 border-gray-200 focus:border-win11-bg-accent text-gray-800'
            )}
          />
        </div>

        <div className="flex-1 flex flex-col gap-2">
          <span className="text-xs font-medium opacity-60 ml-1">
            {t('settings_page.time_unit')}
          </span>
          <div className="flex gap-2">
            {UNITS.map((unit) => (
              <button
                key={unit}
                onClick={() => onUnitChange(unit)}
                className={clsx(
                  'flex-1 py-2.5 rounded-lg border transition-all text-xs font-semibold capitalize',
                  settings.auto_delete_unit === unit
                    ? 'bg-win11-bg-accent text-white border-win11-bg-accent'
                    : isDark
                      ? 'bg-white/5 border-white/10 text-gray-400 hover:bg-white/10'
                      : 'bg-gray-50 border-gray-200 text-gray-600 hover:bg-gray-100'
                )}
              >
                {t(`settings_page.history.${unit}`)}
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="mt-4 p-3 rounded-lg bg-win11-bg-accent/5 border border-win11-bg-accent/10">
        <p className="text-[11px] leading-relaxed opacity-70">
          {settings.auto_delete_interval === 0 ? (
            <span className="font-medium">{t('settings_page.auto_delete_disabled')}</span>
          ) : (
            <>
              {t('settings_page.auto_delete_enabled', {
                count: settings.auto_delete_interval,
                unit: t(`settings_page.history.${settings.auto_delete_unit}`),
              })}
            </>
          )}
        </p>
      </div>
    </SectionCard>
  )
}
