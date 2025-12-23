import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow, Window } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { clsx } from 'clsx'

import type { UserSettings, CustomKaomoji } from './types/clipboard'
import { FeaturesSection } from './components/FeaturesSection'

const DEFAULT_SETTINGS: UserSettings = {
  theme_mode: 'system',
  dark_background_opacity: 0.7,
  light_background_opacity: 0.7,
  enable_smart_actions: true,
  enable_ui_polish: true,
  custom_kaomojis: [],
}

type ThemeMode = 'system' | 'dark' | 'light'

type BooleanSettingKey = {
    [K in keyof UserSettings]: UserSettings[K] extends boolean ? K : never
}[keyof UserSettings]

/**
 * Determines if dark mode should be active based on theme mode setting
 */
function useThemeMode(themeMode: ThemeMode): boolean {
  const [systemPrefersDark, setSystemPrefersDark] = useState(() => {
    if (globalThis.matchMedia) {
      return globalThis.matchMedia('(prefers-color-scheme: dark)').matches
    }
    return true
  })

  useEffect(() => {
    const mediaQuery = globalThis.matchMedia('(prefers-color-scheme: dark)')
    const handleChange = (e: MediaQueryListEvent) => {
      setSystemPrefersDark(e.matches)
    }
    mediaQuery.addEventListener('change', handleChange)
    return () => mediaQuery.removeEventListener('change', handleChange)
  }, [])

  if (themeMode === 'dark') return true
  if (themeMode === 'light') return false
  return systemPrefersDark
}

// --- Icons Components ---
const MonitorIcon = () => (
  <svg
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <rect width="20" height="14" x="2" y="3" rx="2" />
    <line x1="8" x2="16" y1="21" y2="21" />
    <line x1="12" x2="12" y1="17" y2="21" />
  </svg>
)

const MoonIcon = () => (
  <svg
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" />
  </svg>
)

const SunIcon = () => (
  <svg
    width="24"
    height="24"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <circle cx="12" cy="12" r="4" />
    <path d="M12 2v2" />
    <path d="M12 20v2" />
    <path d="m4.93 4.93 1.41 1.41" />
    <path d="m17.66 17.66 1.41 1.41" />
    <path d="M2 12h2" />
    <path d="M20 12h2" />
    <path d="m6.34 17.66-1.41 1.41" />
    <path d="m19.07 4.93-1.41 1.41" />
  </svg>
)

const ResetIcon = () => (
  <svg
    width="16"
    height="16"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
    strokeLinecap="round"
    strokeLinejoin="round"
  >
    <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
    <path d="M3 3v5h5" />
  </svg>
)

/**
 * Settings App Component - Configuration UI for Win11 Clipboard History
 */
function SettingsApp() {
  const [settings, setSettings] = useState<UserSettings>(DEFAULT_SETTINGS)
  const [isLoading, setIsLoading] = useState(true)
  const [isSaving, setIsSaving] = useState(false)
  const [saveMessage, setSaveMessage] = useState<string | null>(null)
  
  // Custom Kaomoji State
  const [newKaomoji, setNewKaomoji] = useState('')

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
        setSettings(loadedSettings)
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
      mainWindow.hide().catch(console.error)
      unlistenClosePromise.then((unlisten) => unlisten())
      unlistenSettingsPromise.then((unlisten) => unlisten())
    }
  }, [])

  // Save settings with debounce-like behavior
  const saveSettings = useCallback(async (newSettings: UserSettings) => {
    setIsSaving(true)
    setSaveMessage(null)

    try {
      await invoke('set_user_settings', { newSettings })
      setSaveMessage('Saved')
      setTimeout(() => setSaveMessage(null), 2000)
    } catch (err) {
      console.error('Failed to save settings:', err)
      setSaveMessage('Error saving')
    } finally {
      setIsSaving(false)
    }
  }, [])
  
  // Centralized settings update helper
  const updateSettings = useCallback((partial: Partial<UserSettings>) => {
      setSettings(prev => {
          const next = { ...prev, ...partial }
          saveSettings(next)
          return next
      })
  }, [saveSettings])

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

  // Commit opacity changes to disk (called on mouseUp/touchEnd)
  const commitOpacityChange = () => {
    saveSettings(settings)
  }

  // Handle Feature Toggles
  const handleToggle = (key: BooleanSettingKey) => {
      // Type safe toggle
      updateSettings({ [key]: !settings[key] } as Partial<UserSettings>)
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

  const removeCustomKaomojiAt = useCallback((index: number) => {
      const newList = settings.custom_kaomojis.filter((_, i) => i !== index)
      updateSettings({ custom_kaomojis: newList })
  }, [settings.custom_kaomojis, updateSettings])

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
          <span className="text-xs opacity-60 font-medium">Loading preferences...</span>
        </div>
      </div>
    )
  }

  return (
    <div
      className={clsx(
        'h-screen w-screen flex flex-col font-sans select-none',
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
          <h1 className="text-2xl font-bold tracking-tight">Personalization</h1>
          <p className={clsx('text-sm mt-1', isDark ? 'text-gray-400' : 'text-gray-500')}>
            Customize the look and feel of your clipboard history
          </p>
        </div>

        {/* Status Indicator */}
        <div className="h-8 flex items-center justify-end min-w-[100px]">
          {(isSaving || saveMessage) && (
            <div
              className={clsx(
                'flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-medium animate-in fade-in slide-in-from-right-4 duration-300',
                saveMessage?.includes('Error')
                  ? 'bg-red-500/10 text-red-500'
                  : isDark
                    ? 'bg-white/10 text-white'
                    : 'bg-black/5 text-black'
              )}
            >
              {isSaving && (
                <div className="w-3 h-3 border-2 border-current border-t-transparent rounded-full animate-spin" />
              )}
              {saveMessage || 'Saving...'}
            </div>
          )}
        </div>
      </header>

      {/* Content */}
      <main className="flex-1 overflow-y-auto px-8 pb-8 space-y-6 custom-scrollbar">
        {/* Theme Selection Card */}
        <section
          className={clsx(
            'rounded-xl p-6 border shadow-sm transition-all',
            isDark ? 'bg-win11-bg-secondary border-white/5' : 'bg-white border-gray-200/60'
          )}
        >
          <div className="flex items-center gap-3 mb-5">
            <div className={clsx('p-2 rounded-lg', isDark ? 'bg-white/5' : 'bg-gray-100')}>
              {settings.theme_mode === 'dark' ? (
                <MoonIcon />
              ) : settings.theme_mode === 'light' ? (
                <SunIcon />
              ) : (
                <MonitorIcon />
              )}
            </div>
            <h2 className="text-base font-semibold">Appearance</h2>
          </div>

          <div className="grid grid-cols-3 gap-4">
            {(['system', 'light', 'dark'] as ThemeMode[]).map((mode) => (
              <button
                key={mode}
                onClick={() => handleThemeModeChange(mode)}
                className={clsx(
                  'group relative flex flex-col items-center gap-3 p-4 rounded-xl border-2 transition-all duration-200 outline-none focus:ring-2 focus:ring-win11-bg-accent/50',
                  settings.theme_mode === mode
                    ? 'border-win11-bg-accent bg-win11-bg-accent/5'
                    : isDark
                      ? 'border-transparent hover:bg-white/5 hover:border-white/10'
                      : 'border-transparent hover:bg-gray-50 hover:border-gray-200'
                )}
              >
                {/* Visual Representation of Theme */}
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

                <div className="flex items-center gap-2">
                  <span
                    className={clsx(
                      'text-sm font-medium capitalize',
                      settings.theme_mode === mode
                        ? 'text-win11-bg-accent'
                        : isDark
                          ? 'text-gray-300'
                          : 'text-gray-700'
                    )}
                  >
                    {mode === 'system' ? 'System' : mode}
                  </span>
                </div>

                {/* Radio Circle Indicator */}
                <div
                  className={clsx(
                    'absolute top-3 right-3 w-4 h-4 rounded-full border flex items-center justify-center transition-colors',
                    settings.theme_mode === mode
                      ? 'border-win11-bg-accent bg-win11-bg-accent'
                      : isDark
                        ? 'border-gray-600'
                        : 'border-gray-300'
                  )}
                >
                  {settings.theme_mode === mode && (
                    <div className="w-1.5 h-1.5 rounded-full bg-white" />
                  )}
                </div>
              </button>
            ))}
          </div>
        </section>

        {/* Transparency Section */}
        <section
          className={clsx(
            'rounded-xl border shadow-sm overflow-hidden',
            isDark ? 'bg-win11-bg-secondary border-white/5' : 'bg-white border-gray-200/60'
          )}
        >
          <div className="p-6 border-b border-inherit">
            <h2 className="text-base font-semibold mb-1">Window Transparency</h2>
            <p className={clsx('text-xs', isDark ? 'text-gray-400' : 'text-gray-500')}>
              Control the backdrop opacity intensity
            </p>
          </div>

          <div className="p-6 space-y-8">
            {/* Dark Mode Slider */}
            <div className="space-y-4">
              <div className="flex justify-between items-center">
                <label htmlFor="dark-opacity" className="text-sm font-medium">
                  Dark Mode Opacity
                </label>
                <div
                  className={clsx(
                    'px-2 py-1 rounded text-xs font-mono font-medium',
                    isDark ? 'bg-black/20' : 'bg-gray-100'
                  )}
                >
                  {Math.round(settings.dark_background_opacity * 100)}%
                </div>
              </div>
              <input
                id="dark-opacity"
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={settings.dark_background_opacity}
                onChange={(e) => handleDarkOpacityChange(Number.parseFloat(e.target.value))}
                onMouseUp={commitOpacityChange}
                onTouchEnd={commitOpacityChange}
                className="w-full h-1.5 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700 accent-win11-bg-accent"
              />
            </div>

            {/* Light Mode Slider */}
            <div className="space-y-4">
              <div className="flex justify-between items-center">
                <label htmlFor="light-opacity" className="text-sm font-medium">
                  Light Mode Opacity
                </label>
                <div
                  className={clsx(
                    'px-2 py-1 rounded text-xs font-mono font-medium',
                    isDark ? 'bg-black/20' : 'bg-gray-100'
                  )}
                >
                  {Math.round(settings.light_background_opacity * 100)}%
                </div>
              </div>
              <input
                id="light-opacity"
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={settings.light_background_opacity}
                onChange={(e) => handleLightOpacityChange(Number.parseFloat(e.target.value))}
                onMouseUp={commitOpacityChange}
                onTouchEnd={commitOpacityChange}
                className="w-full h-1.5 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700 accent-win11-bg-accent"
              />
            </div>
          </div>
        </section>

        {/* Custom Kaomoji Section */}
        <section
          className={clsx(
            'rounded-xl border shadow-sm overflow-hidden',
            isDark ? 'bg-win11-bg-secondary border-white/5' : 'bg-white border-gray-200/60'
          )}
        >
             <div className="p-6 border-b border-inherit">
                <h2 className="text-base font-semibold mb-1">Custom Kaomoji</h2>
                <p className={clsx('text-xs', isDark ? 'text-gray-400' : 'text-gray-500')}>
                    Add your own personal kaomojis to the collection
                </p>
            </div>
            
            <div className="p-6 space-y-6">
                {/* Add New */}
                <div className="flex gap-2">
                    <input 
                        type="text" 
                        value={newKaomoji}
                        onChange={(e) => setNewKaomoji(e.target.value)}
                        placeholder="( ˘ ³˘)♥"
                        className={clsx(
                            "flex-1 px-3 py-2 rounded-md border text-sm focus:outline-none focus:ring-2 focus:ring-win11-bg-accent/50 transition-all",
                            isDark 
                             ? "bg-white/5 border-white/10 text-white placeholder-gray-500" 
                             : "bg-gray-50 border-gray-200 text-gray-900 placeholder-gray-400"
                        )}
                        onKeyDown={(e) => {
                            if (e.key === 'Enter') {
                                addCustomKaomoji()
                            }
                        }}
                    />
                    <button 
                        onClick={addCustomKaomoji}
                        className="px-4 py-2 bg-win11-bg-accent text-white rounded-md text-sm font-medium hover:opacity-90 active:scale-95 transition-all"
                    >
                        Add
                    </button>
                </div>

                {/* List */}
                {settings.custom_kaomojis.length > 0 ? (
                    <div className="grid grid-cols-2 md:grid-cols-3 gap-2 max-h-48 overflow-y-auto custom-scrollbar pr-1">
                        {settings.custom_kaomojis.map((item, idx) => (
                            <div 
                                key={idx}
                                className={clsx(
                                    "group flex items-center justify-between px-3 py-2 rounded-md border transition-colors",
                                    isDark 
                                     ? "bg-white/5 border-white/10" 
                                     : "bg-gray-50 border-gray-200"
                                )}
                            >
                                <span className="font-mono text-sm truncate mr-2" title={item.text}>{item.text}</span>
                                <button
                                    onClick={() => removeCustomKaomojiAt(idx)}
                                    className="opacity-0 group-hover:opacity-100 p-1 text-red-500 hover:bg-red-500/10 rounded transition-all"
                                    title="Delete"
                                >
                                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>
                                </button>
                            </div>
                        ))}
                    </div>
                ) : (
                    <div className={clsx("text-center py-4 text-sm italic opacity-60", isDark ? "text-gray-500" : "text-gray-400")}>
                        No custom kaomojis yet
                    </div>
                )}
            </div>
        </section>

        {/* Features Section */}
        <FeaturesSection 
            settings={settings}
            isDark={isDark}
            onToggle={handleToggle}
        />

        {/* Reset Section */}
        <div className="flex justify-end pt-2">
          <button
            onClick={() => {
              setSettings(DEFAULT_SETTINGS)
              saveSettings(DEFAULT_SETTINGS)
            }}
            className={clsx(
              'flex items-center gap-2 px-4 py-2 text-sm font-medium rounded-lg transition-all',
              'hover:bg-red-50 hover:text-red-600',
              isDark ? 'text-gray-400 hover:bg-red-500/10 hover:text-red-400' : 'text-gray-500'
            )}
          >
            <ResetIcon />
            Reset to defaults
          </button>
        </div>
      </main>

      {/* Footer */}
      <footer
        className={clsx(
          'px-8 py-5 border-t flex justify-end',
          isDark ? 'border-white/5 bg-win11-bg-secondary/50' : 'border-gray-200 bg-gray-50'
        )}
      >
        <button
          onClick={handleClose}
          className="px-8 py-2.5 bg-win11-bg-accent hover:opacity-90 active:scale-95 text-white rounded-lg text-sm font-semibold shadow-sm transition-all"
        >
          Done
        </button>
      </footer>
    </div>
  )
}

export default SettingsApp
