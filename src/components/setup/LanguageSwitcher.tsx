import { clsx } from 'clsx'
import { useTranslation } from 'react-i18next'
import { changeLanguage, type LangCode } from '../../i18n/config'
import { invoke } from '@tauri-apps/api/core'

interface LanguageSwitcherProps {
  readonly isDark: boolean
}

/**
 * The EN/فارسی pill in the wizard's top corner. Persisting the choice
 * goes through the backend `set_app_language` command so the main window
 * and settings pick it up too.
 * / دکمهٔ زبان گوشهٔ بالا؛ انتخاب زبان با فرمان بک‌اند ذخیره می‌شود تا
 * بقیهٔ پنجره‌ها هم آن را ببینند.
 */
export function LanguageSwitcher({ isDark }: LanguageSwitcherProps) {
  const { i18n, t } = useTranslation()

  const handleLanguageChange = async (language: LangCode) => {
    await changeLanguage(language)
    await invoke('set_app_language', { lang: language })
  }

  return (
    <div
      className="absolute top-4 end-4 z-10 flex rounded-lg border p-1 shadow-sm backdrop-blur-md"
      role="group"
      aria-label={t('setup.language_selector')}
    >
      {(['en', 'fa'] as const).map((language) => {
        const selected = i18n.language === language
        return (
          <button
            key={language}
            type="button"
            aria-pressed={selected}
            onClick={() => void handleLanguageChange(language)}
            className={clsx(
              'min-w-16 rounded-md px-3 py-1.5 text-xs font-semibold transition-all',
              selected
                ? 'bg-win11-bg-accent text-white shadow-sm'
                : isDark
                  ? 'text-win11-text-secondary hover:bg-white/10'
                  : 'text-win11Light-text-secondary hover:bg-black/5'
            )}
          >
            {language === 'fa' ? 'فارسی' : 'English'}
          </button>
        )
      })}
    </div>
  )
}
