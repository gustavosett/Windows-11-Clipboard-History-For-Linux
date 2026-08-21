import { useState, useEffect, useCallback, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow, Window } from '@tauri-apps/api/window'
import { emit, listen } from '@tauri-apps/api/event'
import { clsx } from 'clsx'
import { useTranslation } from 'react-i18next'

import type { BooleanSettingKey, CustomKaomoji, UserSettings } from './types/clipboard'
import { FeaturesSection } from './components/FeaturesSection'
import { AppearanceSection } from './components/settings/AppearanceSection'
import { AutoDeleteSection } from './components/settings/AutoDeleteSection'
import { HistorySettingsSection } from './components/settings/HistorySettingsSection'
import { KaomojiSection } from './components/settings/KaomojiSection'
import { KeyboardShortcutsSection } from './components/settings/KeyboardShortcutsSection'
import { ResetSection } from './components/settings/ResetSection'
import { TransparencySection } from './components/settings/TransparencySection'
import { UiScaleSection } from './components/settings/UiScaleSection'
import { PrivacySection } from './components/settings/PrivacySection'
import { useSystemThemePreference } from './utils/systemTheme'
import { useLanguageEffect } from './i18n/useLanguage'
import { changeLanguage } from './i18n/config'
import { useRenderingEnv } from './hooks/useRenderingEnv'
import { DEFAULT_SETTINGS } from './utils/defaultSettings'

const MIN_HISTORY_SIZE = 1
const MAX_HISTORY_SIZE = 2_000

type ThemeMode = 'system' | 'dark' | 'light'

/**
 * Maps theme mode setting to actual dark mode state.
 * For 'system' mode, delegates to the shared useSystemThemePreference hook.
 */
function useThemeMode(themeMode: ThemeMode): boolean {
  const systemPrefersDark = useSystemThemePreference()

  if (themeMode === 'dark') return true
  if (themeMode === 'light') return false
  return systemPrefersDark
}

/**
 * Settings App Component - Configuration UI for Windows 11 Style Clipboard History Manager.
 *
 * Owns the settings state and delegates each visual section to a dedicated
 * component under `components/settings/` (single-responsibility sections).
 */
function SettingsApp() {
  const [settings, setSettings] = useState<UserSettings>(DEFAULT_SETTINGS)
  const [isLoading, setIsLoading] = useState(true)
  const [isSaving, setIsSaving] = useState(false)
  const [saveMessage, setSaveMessage] = useState<string | null>(null)

  // Custom Kaomoji State
  const [newKaomoji, setNewKaomoji] = useState('')

  // Rendering environment (NVIDIA / AppImage detection)
  const renderingEnv = useRenderingEnv()

  const { t, i18n } = useTranslation()
  // i18n
  useLanguageEffect(i18n)

  // Apply theme to settings window itself
  const isDark = useThemeMode(settings.theme_mode)

  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }, [isDark])

  // Load settings on mount and show main window for preview
  useEffect(() => {
    invoke<UserSettings>('get_user_settings')
      .then((loadedSettings) => {
        setSettings({ ...DEFAULT_SETTINGS, ...loadedSettings })
        setIsLoading(false)
      })
      .catch((err) => {
        console.error('Failed to load settings:', err)
        setIsLoading(false)
      })

    // Show the main clipboard window for live preview
    const mainWindow = new Window('main')
    mainWindow.show().catch(console.error)

    // Prevent window close, just hide it instead
    const currentWindow = getCurrentWindow()
    const unlistenClosePromise = currentWindow.onCloseRequested(async (event) => {
      event.preventDefault()
      await currentWindow.hide()
    })

    // Listen for settings changes (in case another settings window is open)
    const unlistenSettingsPromise = listen<UserSettings>('app-settings-changed', (event) => {
      setSettings(event.payload)
    })

    // Hide main window when settings window closes
    return () => {
      void mainWindow.hide().catch(console.error)
      void unlistenClosePromise.then((unlisten) => {
        unlisten()
      })
      void unlistenSettingsPromise.then((unlisten) => {
        unlisten()
      })
    }
  }, [])

  // Save settings with debounce-like behavior
  const saveSettings = useCallback(async (newSettings: UserSettings) => {
    setIsSaving(true)
    setSaveMessage(null)

    try {
      await invoke('set_user_settings', { newSettings })
      setSaveMessage(t('settings_page.saved'))
      setTimeout(() => setSaveMessage(null), 2000)
    } catch (err) {
      console.error('Failed to save settings:', err)
      setSaveMessage(t('settings_page.save_error'))
    } finally {
      setIsSaving(false)
    }
  }, [t])

  // Centralized settings update helper.
  // The merged snapshot is computed outside the state updater (updaters must
  // stay pure); a ref mirrors the latest state so rapid toggles never save a
  // stale snapshot.
  // به‌روزرسان متمرکز تنظیمات. تصویر ادغام‌شده خارج از updater محاسبه
  // می‌شود (updater باید خالص بماند)؛ یک ref آخرین state را آینه می‌کند تا
  // تغییرات سریع هرگز تصویر کهنه‌ای ذخیره نکنند.
  const settingsRef = useRef(settings)
  useEffect(() => {
    settingsRef.current = settings
  }, [settings])

  const updateSettings = useCallback(
    (partial: Partial<UserSettings>) => {
      const next = { ...settingsRef.current, ...partial }
      setSettings(next)
      void saveSettings(next)
    },
    [saveSettings]
  )

  // Handle theme mode change
  const handleThemeModeChange = (mode: ThemeMode) => {
    updateSettings({ theme_mode: mode })
  }

  // Handle dark opacity change (visual only, no disk I/O)
  const handleDarkOpacityChange = (value: number) => {
    setSettings((prev) => ({ ...prev, dark_background_opacity: value }))
  }

  // Handle light opacity change (visual only, no disk I/O)
  const handleLightOpacityChange = (value: number) => {
    setSettings((prev) => ({ ...prev, light_background_opacity: value }))
  }

  // Commit opacity changes to disk (called on mouseUp/touchEnd/keyUp)
  const commitOpacityChange = () => {
    void saveSettings(settings)
  }

  const handleAutoDeleteValueChange = (value: string) => {
    // Only allow positive integers or 0
    const num = Number.parseInt(value)
    if (value === '' || (Number.isInteger(num) && num >= 0)) {
      const interval = value === '' ? 0 : num
      const newSettings = { ...settings, auto_delete_interval: interval }
      setSettings(newSettings)
      void saveSettings(newSettings)
    }
  }

  const handleAutoDeleteUnitChange = (unit: UserSettings['auto_delete_unit']) => {
    const newSettings = { ...settings, auto_delete_unit: unit }
    setSettings(newSettings)
    void saveSettings(newSettings)
  }

  const handleUiScaleChange = (value: number) => {
    setSettings((prev) => ({ ...prev, ui_scale: value }))
  }

  const handleMaxHistoryChange = (value: number) => {
    updateSettings({ max_history_size: value })
  }

  // Handle Feature Toggles
  const handleToggle = (key: BooleanSettingKey) => {
    // Type safe toggle
    updateSettings({ [key]: !settings[key] })
  }

  const handleLanguageChange = (lang: 'en' | 'fa') => {
    updateSettings({ language: lang })
    void changeLanguage(lang)
    invoke('set_app_language', { lang }).catch(console.error)
  }

  // Custom Kaomoji Handlers
  const addCustomKaomoji = useCallback(() => {
    const val = newKaomoji.trim()
    if (!val) return

    const newItem: CustomKaomoji = {
      text: val,
      category: 'Custom',
      keywords: ['custom'],
    }

    updateSettings({ custom_kaomojis: [...settings.custom_kaomojis, newItem] })
    setNewKaomoji('')
  }, [newKaomoji, settings.custom_kaomojis, updateSettings])

  const removeCustomKaomojiAt = useCallback(
    (index: number) => {
      const newList = settings.custom_kaomojis.filter((_, i) => i !== index)
      updateSettings({ custom_kaomojis: newList })
    },
    [settings.custom_kaomojis, updateSettings]
  )

  const handleReset = useCallback(async () => {
    setSettings(DEFAULT_SETTINGS)
    await saveSettings(DEFAULT_SETTINGS)
    // Reset first run state to show setup wizard
    await invoke('reset_first_run')
    // Emit event to show wizard in main window
    await emit('show-setup-wizard')
  }, [saveSettings])

  // Handle window close
  const handleClose = async () => {
    try {
      await getCurrentWindow().hide()
    } catch (err) {
      console.error('Failed to close window:', err)
    }
  }

  if (isLoading) {
    return (
      <div
        className={clsx(
          'h-screen w-screen flex items-center justify-center select-none',
          isDark
            ? 'bg-win11-bg-primary text-win11-text-primary'
            : 'bg-win11Light-bg-primary text-win11Light-text-primary'
        )}
      >
        <div className="flex flex-col items-center gap-3">
          <div className="w-6 h-6 border-2 border-win11-bg-accent border-t-transparent rounded-full animate-spin" />
          <span className="text-xs opacity-60 font-medium">{t('settings_page.loading_prefs')}</span>
        </div>
      </div>
    )
  }

  return (
    <div
      className={clsx(
        'h-screen w-screen flex flex-col font-sans select-none animate-window-in',
        isDark
          ? 'bg-win11-bg-primary text-win11-text-primary'
          : 'bg-[#f0f3f9] text-win11Light-text-primary' // Slightly better light gray background
      )}
    >
      {/* Header */}
      <header
        className={clsx(
          'flex items-center justify-between px-8 py-6 flex-shrink-0',
          'transition-colors duration-200'
        )}
      >
        <div>
          <h1 className="text-2xl font-bold tracking-tight">{t('settings_page.personalization')}</h1>
          <p className={clsx('text-sm mt-1', isDark ? 'text-gray-400' : 'text-gray-500')}>
            {t('settings_page.personalization_desc')}
          </p>
        </div>

        {/* Status Indicator */}
        <div className="h-8 flex items-center justify-end min-w-[100px]">
          {(isSaving || saveMessage) && (
            <div
              className={clsx(
                'flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-medium',
                saveMessage === t('settings_page.save_error')
                  ? 'bg-red-500/10 text-red-500'
                  : isDark
                    ? 'bg-white/10 text-white'
                    : 'bg-black/5 text-black'
              )}
            >
              {isSaving && (
                <div className="w-3 h-3 border-2 border-current border-t-transparent rounded-full animate-spin" />
              )}
              {saveMessage || t('settings_page.saving')}
            </div>
          )}
        </div>
      </header>

      {/* Content */}
      <main className="flex-1 overflow-y-auto px-8 pb-8 space-y-6 custom-scrollbar">
        <AppearanceSection
          settings={settings}
          isDark={isDark}
          onThemeModeChange={handleThemeModeChange}
          onToggle={handleToggle}
          onLanguageChange={handleLanguageChange}
        />

        <AutoDeleteSection
          settings={settings}
          isDark={isDark}
          onIntervalChange={handleAutoDeleteValueChange}
          onUnitChange={handleAutoDeleteUnitChange}
        />

        <TransparencySection
          settings={settings}
          isDark={isDark}
          renderingEnv={renderingEnv}
          onDarkOpacityChange={handleDarkOpacityChange}
          onLightOpacityChange={handleLightOpacityChange}
          onCommit={commitOpacityChange}
        />

        <UiScaleSection
          settings={settings}
          isDark={isDark}
          onScaleChange={handleUiScaleChange}
          onCommit={commitOpacityChange}
        />

        <HistorySettingsSection
          settings={settings}
          isDark={isDark}
          min={MIN_HISTORY_SIZE}
          max={MAX_HISTORY_SIZE}
          onMaxHistoryChange={handleMaxHistoryChange}
        />

        <KaomojiSection
          settings={settings}
          isDark={isDark}
          newKaomoji={newKaomoji}
          onNewKaomojiChange={setNewKaomoji}
          onAdd={addCustomKaomoji}
          onRemove={removeCustomKaomojiAt}
        />

        <PrivacySection settings={settings} isDark={isDark} onToggle={handleToggle} />

        {/* Features Section */}
        <FeaturesSection settings={settings} isDark={isDark} onToggle={handleToggle} />

        {/* Keyboard Shortcuts Section */}
        <KeyboardShortcutsSection isDark={isDark} />

        {/* Reset Section */}
        <ResetSection isDark={isDark} onReset={() => void handleReset()} />
      </main>

      {/* Footer */}
      <footer
        className={clsx(
          'px-8 py-5 border-t flex justify-end',
          isDark ? 'border-white/5 bg-win11-bg-secondary/50' : 'border-gray-200 bg-gray-50'
        )}
      >
        <button
          onClick={() => void handleClose()}
          className="px-8 py-2.5 bg-win11-bg-accent hover:opacity-90 active:scale-95 text-white rounded-lg text-sm font-semibold shadow-sm transition-all"
        >
          {i18n.t('settings_page.done')}
        </button>
      </footer>
    </div>
  )
}

export default SettingsApp
