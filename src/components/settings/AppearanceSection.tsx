import { clsx } from 'clsx'
import { useTranslation } from 'react-i18next'
import type { BooleanSettingKey, ThemeMode, UserSettings } from '../../types/clipboard'
import { Switch } from '../Switch'
import { MonitorIcon, MoonIcon, SunIcon } from './icons'
import { SectionCard } from './SectionCard'

interface AppearanceSectionProps {
  settings: UserSettings
  isDark: boolean
  onThemeModeChange: (mode: ThemeMode) => void
  onToggle: (key: BooleanSettingKey) => void
  onLanguageChange: (lang: 'en' | 'fa') => void
}

const THEME_MODES: ThemeMode[] = ['system', 'light', 'dark']

/** Theme / language / tray-icon appearance settings. */
export function AppearanceSection({
  settings,
  isDark,
  onThemeModeChange,
  onToggle,
  onLanguageChange,
}: AppearanceSectionProps) {
  const { t } = useTranslation()

  return (
    <SectionCard
      title={t('settings_page.appearance')}
      dividedHeader={false}
      isDark={isDark}
      icon={
        settings.theme_mode === 'dark' ? (
          <MoonIcon />
        ) : settings.theme_mode === 'light' ? (
          <SunIcon />
        ) : (
          <MonitorIcon />
        )
      }
    >
      <div className="grid grid-cols-3 gap-4">
        {THEME_MODES.map((mode) => {
          const selected = settings.theme_mode === mode
          return (
            <button
              key={mode}
              onClick={() => onThemeModeChange(mode)}
              className={clsx(
                'group relative flex flex-col items-center gap-3 p-4 rounded-xl border-2 transition-all duration-200 outline-none focus:ring-2 focus:ring-win11-bg-accent/50',
                selected
                  ? 'border-win11-bg-accent bg-win11-bg-accent/5'
                  : isDark
                    ? 'border-transparent hover:bg-white/5 hover:border-white/10'
                    : 'border-transparent hover:bg-gray-50 hover:border-gray-200'
              )}
            >
              {/* Visual representation of the theme */}
              <div
                className={clsx(
                  'w-full aspect-[16/10] rounded-lg shadow-sm flex overflow-hidden border',
                  isDark ? 'border-white/10' : 'border-gray-200'
                )}
              >
                {mode === 'system' && (
                  <>
                    <div className="flex-1 bg-[#f3f3f3]" />
                    <div className="flex-1 bg-[#202020]" />
                  </>
                )}
                {mode === 'light' && <div className="flex-1 bg-[#f3f3f3]" />}
                {mode === 'dark' && <div className="flex-1 bg-[#202020]" />}
              </div>

              <span
                className={clsx(
                  'text-sm font-medium capitalize',
                  selected ? 'text-win11-bg-accent' : isDark ? 'text-gray-300' : 'text-gray-700'
                )}
              >
                {t(`settings_page.theme.${mode}`)}
              </span>

              {/* Radio circle indicator */}
              <div
                className={clsx(
                  'absolute top-3 right-3 w-4 h-4 rounded-full border flex items-center justify-center transition-colors',
                  selected
                    ? 'border-win11-bg-accent bg-win11-bg-accent'
                    : isDark
                      ? 'border-gray-600'
                      : 'border-gray-300'
                )}
              >
                {selected && <div className="w-1.5 h-1.5 rounded-full bg-white" />}
              </div>
            </button>
          )
        })}
      </div>

      <div className={clsx('mt-6 pt-6 border-t', isDark ? 'border-white/5' : 'border-gray-100')}>
        <div className="flex items-center justify-between">
          <div>
            <div className="text-sm font-medium">{t('settings_page.features.dynamic_tray')}</div>
            <div className={clsx('text-xs mt-0.5', isDark ? 'text-gray-400' : 'text-gray-500')}>
              {t('settings_page.features.dynamic_tray_desc')}
            </div>
          </div>
          <Switch
            checked={settings.enable_dynamic_tray_icon}
            onChange={() => onToggle('enable_dynamic_tray_icon')}
            isDark={isDark}
          />
        </div>
      </div>

      <div className={clsx('mt-6 pt-6 border-t', isDark ? 'border-white/5' : 'border-gray-100')}>
        <div className="text-sm font-medium mb-3">{t('settings_page.language.label')}</div>
        <div className="grid grid-cols-2 gap-3">
          {[
            { id: 'en' as const, label: t('settings_page.language.english') },
            { id: 'fa' as const, label: t('settings_page.language.persian') },
          ].map((lang) => (
            <button
              key={lang.id}
              onClick={() => onLanguageChange(lang.id)}
              className={clsx(
                'px-4 py-2.5 rounded-lg border text-sm font-medium transition-all',
                settings.language === lang.id
                  ? 'border-win11-bg-accent bg-win11-bg-accent/10 text-win11-bg-accent'
                  : isDark
                    ? 'border-white/10 hover:bg-white/5'
                    : 'border-gray-200 hover:bg-gray-50'
              )}
            >
              {lang.label}
            </button>
          ))}
        </div>
      </div>
    </SectionCard>
  )
}
