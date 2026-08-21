import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { clsx } from 'clsx'
import { useTranslation } from 'react-i18next'
import { useLanguageEffect } from '../i18n/useLanguage'
import { useAutostart } from '../hooks/useAutostart'
import { useSystemThemePreference } from '../utils/systemTheme'
import { LanguageSwitcher } from './setup/LanguageSwitcher'
import { WelcomeStep } from './setup/WelcomeStep'
import { PermissionsStep } from './setup/PermissionsStep'
import { ShortcutStep } from './setup/ShortcutStep'
import { AutostartStep } from './setup/AutostartStep'
import { DoneStep } from './setup/DoneStep'
import { useSetupChecks } from './setup/useSetupChecks'

interface SetupWizardProps {
  readonly onComplete: () => void
}

const TOTAL_STEPS = 5

/**
 * First-run setup wizard orchestrator. Each step renders itself
 * (see `./setup/*`); this component owns navigation, the shared theme
 * context, and the long-running actions (permission fix, shortcut
 * registration, conflict resolution, autostart).
 * / هماهنگ‌کنندهٔ جادوگر راه‌اندازی اولیه؛ هر گام خودش را رسم می‌کند و
 * این مؤلفه مالک ناوبری، زمینهٔ تم و عملیات طولانی است.
 */
export function SetupWizard({ onComplete }: SetupWizardProps) {
  const { t, i18n } = useTranslation()
  useLanguageEffect(i18n)
  const [step, setStep] = useState(0)
  const [fixing, setFixing] = useState(false)
  const [fixError, setFixError] = useState<string | null>(null)
  const [registeringShortcut, setRegisteringShortcut] = useState(false)
  const [shortcutRegistered, setShortcutRegistered] = useState(false)
  const [showManualInstructions, setShowManualInstructions] = useState(false)
  const [resolvingConflicts, setResolvingConflicts] = useState(false)
  const [conflictsResolved, setConflictsResolved] = useState(false)
  const [conflictError, setConflictError] = useState<string | null>(null)
  const [copied, setCopied] = useState(false)
  const [hoveredButton, setHoveredButton] = useState<string | null>(null)
  const { enableAutostart } = useAutostart()
  const isDark = useSystemThemePreference()

  const { permissions, shortcutTools, conflicts, checkPermissions, checkShortcutTools, checkConflicts } =
    useSetupChecks()

  // Fixed opacity for the wizard (similar to main app default)
  const tertiaryOpacity = 0.85
  const buttonProps = { hoveredButton, setHoveredButton, isDark, tertiaryOpacity }

  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }, [isDark])

  const localizedPermissionError = (error: unknown) => {
    const raw = String(error)
    const code = ['pkexec_missing', 'setfacl_missing', 'permission_fix_failed'].find((candidate) =>
      raw.includes(candidate)
    )
    return code ? t(`setup.${code}`) : t('setup.permission_fix_failed')
  }

  const handleResolveConflicts = async () => {
    setResolvingConflicts(true)
    setConflictError(null)
    try {
      await invoke<string[]>('resolve_conflicts')
      setConflictsResolved(true)
      // Refresh conflict status
      await checkConflicts()
      await checkShortcutTools()
    } catch (e) {
      console.error('Failed to resolve conflicts:', e)
      setConflictError(t('setup.conflicts_failed_detail'))
    } finally {
      setResolvingConflicts(false)
    }
  }

  const handleFixPermissions = async () => {
    setFixing(true)
    setFixError(null)
    try {
      await invoke<string>('fix_permissions_now')
      await checkPermissions()
    } catch (e) {
      console.error('Failed to fix permissions:', e)
      setFixError(localizedPermissionError(e))
    } finally {
      setFixing(false)
    }
  }

  const handleRegisterShortcut = async () => {
    setRegisteringShortcut(true)
    try {
      await invoke<string>('register_de_shortcut')
      setShortcutRegistered(true)
      setTimeout(() => setStep(3), 1500)
    } catch (e) {
      console.error('Failed to register shortcut:', e)
      setShowManualInstructions(true)
    } finally {
      setRegisteringShortcut(false)
    }
  }

  const handleEnableAutostart = async () => {
    await enableAutostart()
    setStep(4)
  }

  const handleComplete = () => {
    // The setup-only `finish_setup` command atomically persists completion.
    // فرمان setup-only با نام `finish_setup` تکمیل را اتمیک ذخیره می‌کند.
    onComplete()
  }

  const copyToClipboard = (text: string) => {
    void navigator.clipboard.writeText(text)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const getProgressDotClass = (i: number) => {
    if (i === step) return 'bg-win11-bg-accent w-5'
    if (i < step)
      return clsx(
        'cursor-pointer',
        isDark
          ? 'bg-win11-text-tertiary hover:bg-win11-text-secondary'
          : 'bg-win11Light-text-secondary hover:bg-win11Light-text-primary'
      )
    return isDark ? 'bg-win11-border' : 'bg-win11Light-border'
  }

  return (
    <div
      className={clsx(
        'relative h-full w-full flex flex-col items-center justify-center p-6 animate-window-in',
        isDark
          ? 'bg-win11-bg-primary text-win11-text-primary'
          : 'bg-win11Light-bg-primary text-win11Light-text-primary'
      )}
    >
      <LanguageSwitcher isDark={isDark} />

      <div className="w-full max-w-sm">
        {step === 0 && <WelcomeStep {...buttonProps} onStart={() => setStep(1)} />}
        {step === 1 && (
          <PermissionsStep
            {...buttonProps}
            permissions={permissions}
            fixError={fixError}
            fixing={fixing}
            onFix={() => void handleFixPermissions()}
            onContinue={() => setStep(2)}
          />
        )}
        {step === 2 && (
          <ShortcutStep
            {...buttonProps}
            shortcutTools={shortcutTools}
            conflicts={conflicts}
            conflictsResolved={conflictsResolved}
            conflictError={conflictError}
            resolvingConflicts={resolvingConflicts}
            registeringShortcut={registeringShortcut}
            shortcutRegistered={shortcutRegistered}
            showManualInstructions={showManualInstructions}
            copied={copied}
            onResolveConflicts={() => void handleResolveConflicts()}
            onRegisterShortcut={() => void handleRegisterShortcut()}
            onShowManualInstructions={() => setShowManualInstructions(true)}
            onCopyCommand={() =>
              copyToClipboard('/usr/bin/windows-11-style-clipboard-history-manager')
            }
            onContinue={() => setStep(3)}
          />
        )}
        {step === 3 && (
          <AutostartStep
            {...buttonProps}
            onEnable={() => void handleEnableAutostart()}
            onSkip={() => setStep(4)}
          />
        )}
        {step === 4 && <DoneStep {...buttonProps} onFinish={handleComplete} />}

        {/* Progress dots / نقطه‌های پیشرفت */}
        <div className="flex justify-center gap-2 mt-8">
          {Array.from({ length: TOTAL_STEPS }, (_, i) => (
            <button
              key={`dot-${i}`}
              onClick={() => i < step && setStep(i)}
              disabled={i >= step}
              aria-label={t('setup.step_label', { step: i + 1 })}
              aria-current={i === step ? 'step' : undefined}
              className={clsx(
                'h-1.5 w-1.5 rounded-full transition-all duration-200',
                getProgressDotClass(i)
              )}
            />
          ))}
        </div>
      </div>
    </div>
  )
}
