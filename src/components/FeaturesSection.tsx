import { clsx } from 'clsx'
import { Switch } from './Switch'
import type { UserSettings } from '../types/clipboard'

const FEATURES = [
  {
    key: 'enable_smart_actions',
    label: 'Smart Actions',
    desc: 'Automatically detect links, colors, and emails.',
  },
  {
    key: 'enable_ui_polish',
    label: 'UI Polish',
    desc: 'Enable animations and compact mode support.',
  },
] as const

type BooleanSettingKey = {
  [K in keyof UserSettings]: UserSettings[K] extends boolean ? K : never
}[keyof UserSettings]

export function FeaturesSection({
  settings,
  isDark,
  onToggle,
}: {
  settings: UserSettings
  isDark: boolean
  onToggle: (key: BooleanSettingKey) => void
}) {
  return (
    <section
      className={clsx(
        'rounded-xl border shadow-sm overflow-hidden',
        isDark ? 'bg-win11-bg-secondary border-white/5' : 'bg-white border-gray-200/60'
      )}
    >
      <div className="p-6 border-b border-inherit">
        <h2 className="text-base font-semibold mb-1">Features</h2>
        <p className={clsx('text-xs', isDark ? 'text-gray-400' : 'text-gray-500')}>
          Customize the functionality of the application
        </p>
      </div>
      <div className="p-6 space-y-6">
        {FEATURES.map((feature) => (
          <div key={feature.key} className="flex items-center justify-between">
            <div>
              <div className="text-sm font-medium">{feature.label}</div>
              <div className={clsx('text-xs', isDark ? 'text-gray-400' : 'text-gray-500')}>
                {feature.desc}
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
