import { clsx } from 'clsx'
import { useTranslation } from 'react-i18next'
import { Switch } from './Switch'
import type { UserSettings, BooleanSettingKey } from '../types/clipboard'

const FEATURES = [
  {
    key: 'enable_smart_actions',
    labelKey: 'features.smart_actions',
    descKey: 'features.smart_actions_desc',
  },
  {
    key: 'enable_ui_polish',
    labelKey: 'features.ui_polish',
    descKey: 'features.ui_polish_desc',
  },
] as const

export function FeaturesSection({
  settings,
  isDark,
  onToggle,
}: {
  settings: UserSettings
  isDark: boolean
  onToggle: (key: BooleanSettingKey) => void
}) {
  const { t } = useTranslation()

  return (
    <section
      className={clsx(
        'rounded-xl border shadow-sm overflow-hidden',
        isDark ? 'bg-win11-bg-secondary border-white/5' : 'bg-white border-gray-200/60'
      )}
    >
      <div className="p-6 border-b border-inherit">
        <h2 className="text-base font-semibold mb-1">{t('features.title')}</h2>
        <p className={clsx('text-xs', isDark ? 'text-gray-400' : 'text-gray-500')}>
          {t('features.description')}
        </p>
      </div>
      <div className="p-6 space-y-6">
        {FEATURES.map((feature) => (
          <div key={feature.key} className="flex items-center justify-between">
            <div>
              <div className="text-sm font-medium">{t(feature.labelKey)}</div>
              <div className={clsx('text-xs', isDark ? 'text-gray-400' : 'text-gray-500')}>
                {t(feature.descKey)}
              </div>
            </div>
            <Switch
              checked={settings[feature.key]}
              onChange={() => onToggle(feature.key)}
              isDark={isDark}
            />
          </div>
        ))}
      </div>
    </section>
  )
}
