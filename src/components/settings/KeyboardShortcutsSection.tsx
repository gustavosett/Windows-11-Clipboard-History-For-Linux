import { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { clsx } from 'clsx'
import { useTranslation } from 'react-i18next'
import { CheckIcon, KeyboardIcon, PlusIcon } from './icons'
import { SectionCard } from './SectionCard'

interface KeyboardShortcutsSectionProps {
  isDark: boolean
}

const SHORTCUT_ITEMS = [
  { keys: 'Super + V', descKey: 'settings_page.shortcut_open_history' },
  { keys: 'Ctrl + Alt + V', descKey: 'settings_page.shortcut_alternative' },
  { keys: 'Super + .', descKey: 'settings_page.shortcut_open_emoji' },
]

/** Desktop-environment shortcut registration panel. */
export function KeyboardShortcutsSection({ isDark }: KeyboardShortcutsSectionProps) {
  const { t } = useTranslation()
  const [registering, setRegistering] = useState(false)
  const [status, setStatus] = useState<'idle' | 'success' | 'error'>('idle')
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  const handleRegister = async () => {
    setRegistering(true)
    setStatus('idle')
    setErrorMessage(null)
    try {
      await invoke<string>('register_de_shortcut')
      setStatus('success')
    } catch (e) {
      console.error('Failed to register shortcuts:', e)
      setStatus('error')
      setErrorMessage(String(e))
    } finally {
      setRegistering(false)
    }
  }

  return (
    <SectionCard
      icon={<KeyboardIcon />}
      title={t('settings_page.keyboard_shortcuts')}
      subtitle={t('settings_page.keyboard_shortcuts_desc')}
      isDark={isDark}
    >
      <div className="space-y-4">
        {/* Shortcut list */}
        <div
          className={clsx(
            'rounded-lg border divide-y text-sm font-mono',
            isDark
              ? 'border-white/10 divide-white/10 bg-black/20'
              : 'border-gray-200 divide-gray-100 bg-gray-50'
          )}
        >
          {SHORTCUT_ITEMS.map(({ keys, descKey }) => (
            <div key={keys} className="flex items-center justify-between px-4 py-2.5">
              <span
                className={clsx(
                  'px-2 py-0.5 rounded text-xs font-semibold',
                  isDark ? 'bg-white/10 text-gray-200' : 'bg-gray-200 text-gray-700'
                )}
              >
                {keys}
              </span>
              <span
                className={clsx('text-xs font-sans', isDark ? 'text-gray-400' : 'text-gray-500')}
              >
                {t(descKey)}
              </span>
            </div>
          ))}
        </div>

        <p className={clsx('text-xs leading-relaxed', isDark ? 'text-gray-500' : 'text-gray-400')}>
          {t('settings_page.shortcut_privacy_note')}
        </p>

        {/* Status feedback */}
        {status === 'success' && (
          <div className="flex items-center gap-2 text-sm text-green-500">
            <CheckIcon />
            {t('settings_page.shortcut_success')}
          </div>
        )}
        {status === 'error' && (
          <div
            className={clsx(
              'text-xs rounded-lg p-3',
              isDark ? 'bg-red-500/10 text-red-400' : 'bg-red-50 text-red-600'
            )}
          >
            <p className="font-medium">{t('settings_page.shortcut_failed')}</p>
            {errorMessage && <p className="mt-1 opacity-80">{errorMessage}</p>}
          </div>
        )}

        {/* Action button */}
        <button
          id="register-shortkeys-btn"
          onClick={() => void handleRegister()}
          disabled={registering}
          className={clsx(
            'flex items-center gap-2 px-5 py-2.5 rounded-lg text-sm font-semibold transition-all',
            'bg-win11-bg-accent text-white hover:opacity-90 active:scale-95',
            'disabled:opacity-50 disabled:cursor-not-allowed disabled:active:scale-100'
          )}
        >
          {registering ? (
            <>
              <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
              {t('settings_page.registering')}
            </>
          ) : (
            <>
              <PlusIcon />
              {t('settings_page.register_shortcuts')}
            </>
          )}
        </button>
      </div>
    </SectionCard>
  )
}
