import { clsx } from 'clsx'
import { useTranslation } from 'react-i18next'
import type { UserSettings } from '../../types/clipboard'
import { SectionCard } from './SectionCard'

interface HistorySettingsSectionProps {
  settings: UserSettings
  isDark: boolean
  min: number
  max: number
  onMaxHistoryChange: (value: number) => void
}

/** History size / retention settings. */
export function HistorySettingsSection({
  settings,
  isDark,
  min,
  max,
  onMaxHistoryChange,
}: HistorySettingsSectionProps) {
  const { t, i18n } = useTranslation()

  return (
    <SectionCard
      title={t('settings_page.history_settings')}
      subtitle={t('settings_page.history_settings_desc')}
      isDark={isDark}
    >
      <div className="flex justify-between items-center">
        <div>
          <label htmlFor="max-history" className="text-sm font-medium">
            {t('settings_page.history.max_size')}
          </label>
          <p className={clsx('text-xs mt-0.5', isDark ? 'text-gray-400' : 'text-gray-500')}>
            {t('settings_page.max_history_desc', { min, max: max.toLocaleString(i18n.language) })}
          </p>
        </div>
        <input
          id="max-history"
          type="number"
          min={min}
          max={max}
          value={settings.max_history_size}
          onChange={(e) => {
            const raw = e.target.value
            const parsed = Number.parseInt(raw, 10)
            // If parsing fails (e.g. empty input), preserve the current setting
            // instead of jumping to the maximum value.
            const safe = Number.isNaN(parsed) ? settings.max_history_size : parsed
            onMaxHistoryChange(Math.max(min, Math.min(max, safe)))
          }}
          className={clsx(
            'w-28 text-right font-mono border rounded-md transition-all focus:outline-none focus:ring-2 focus:ring-win11-bg-accent/50',
            'input-number-compact no-number-spinner',
            isDark
              ? 'bg-white/5 border-white/10 text-white'
              : 'bg-gray-50 border-gray-200 text-gray-900'
          )}
        />
      </div>
    </SectionCard>
  )
}
