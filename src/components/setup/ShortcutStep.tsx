import {
  AlertCircle,
  AlertTriangle,
  CheckCircle,
  Copy,
  Keyboard,
  Settings,
  Zap,
} from 'lucide-react'
import { clsx } from 'clsx'
import { useTranslation } from 'react-i18next'
import { StepHeader } from './StepHeader'
import { StatusCard } from './StatusCard'
import { WizardButton } from './WizardButton'
import type {
  ConflictDetectionResult,
  ShortcutToolsStatus,
  WizardButtonContext,
} from './types'

interface ShortcutStepProps extends WizardButtonContext {
  readonly shortcutTools: ShortcutToolsStatus | null
  readonly conflicts: ConflictDetectionResult | null
  readonly conflictsResolved: boolean
  readonly conflictError: string | null
  readonly resolvingConflicts: boolean
  readonly registeringShortcut: boolean
  readonly shortcutRegistered: boolean
  readonly showManualInstructions: boolean
  readonly copied: boolean
  readonly onResolveConflicts: () => void
  readonly onRegisterShortcut: () => void
  readonly onShowManualInstructions: () => void
  readonly onCopyCommand: () => void
  readonly onContinue: () => void
}

/**
 * Step 2 — DE shortcut registration: auto-register when the desktop
 * environment supports it, conflict auto-resolution when possible, and
 * a manual-instructions fallback with a copyable command otherwise.
 * / گام ۲ — ثبت میانبر میزکار: ثبت خودکار در صورت پشتیبانی، رفع خودکار
 * تداخل، و در غیر این صورت راهنمای دستی با فرمان قابل کپی.
 */
export function ShortcutStep({
  shortcutTools,
  conflicts,
  conflictsResolved,
  conflictError,
  resolvingConflicts,
  registeringShortcut,
  shortcutRegistered,
  showManualInstructions,
  copied,
  onResolveConflicts,
  onRegisterShortcut,
  onShowManualInstructions,
  onCopyCommand,
  onContinue,
  ...buttonContext
}: ShortcutStepProps) {
  const { t } = useTranslation()
  const { isDark } = buttonContext

  const infoCardClass = clsx(
    'p-3 rounded-win11',
    isDark
      ? 'bg-win11-bg-tertiary/50 border border-win11-border-subtle'
      : 'bg-win11Light-bg-tertiary/50 border border-win11Light-border'
  )

  return (
    <div className="animate-step-in">
      <StepHeader
        icon={Keyboard}
        title={t('setup.step_shortcut')}
        subtitle={
          <>
            {t('setup.shortcut_intro_before')}{' '}
            <kbd
              className={clsx(
                'px-2 py-0.5 rounded text-xs font-mono',
                isDark ? 'bg-win11-bg-tertiary' : 'bg-win11Light-bg-tertiary'
              )}
            >
              Super + V
            </kbd>{' '}
            {t('setup.shortcut_intro_after')}
          </>
        }
        isDark={isDark}
      />

      {shortcutTools && (
        <div className={clsx('mb-4', infoCardClass)}>
          <div
            className={clsx(
              'flex items-center gap-2 text-sm',
              isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
            )}
          >
            <Settings className="w-4 h-4" />
            <span>
              {t('setup.detected')}{' '}
              <strong
                className={isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'}
              >
                {shortcutTools.desktop_environment}
              </strong>
            </span>
          </div>
        </div>
      )}

      {/* Conflict warning / هشدار تداخل */}
      {conflicts && conflicts.conflicts.length > 0 && !conflictsResolved && (
        <StatusCard type="warning" isDark={isDark} className="mb-4">
          <AlertCircle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          <div className="flex-1">
            <p className="font-medium mb-1">
              {t('setup.conflicts_detected', { count: conflicts.conflicts.length })}
            </p>
            <p className="text-xs opacity-90 mb-2">
              {t('setup.conflict_detail', {
                owner: conflicts.conflicts[0].owner,
                action: conflicts.conflicts[0].current_action,
              })}
            </p>
            {conflicts.can_auto_resolve && (
              <div className="space-y-1">
                <WizardButton
                  {...buttonContext}
                  id="resolve-conflicts"
                  onClick={onResolveConflicts}
                  disabled={resolvingConflicts}
                >
                  <span className="flex items-center gap-2">
                    <Zap className="w-4 h-4" />
                    {resolvingConflicts ? t('setup.resolving') : t('setup.auto_fix')}
                  </span>
                </WizardButton>
                <p className="text-xs opacity-60">{t('setup.auto_fix_note')}</p>
              </div>
            )}
            {!conflicts.can_auto_resolve && (
              <p className="text-xs opacity-75 mt-1">{t('setup.manual_resolution')}</p>
            )}
          </div>
        </StatusCard>
      )}

      {conflictsResolved && (
        <StatusCard type="success" isDark={isDark} className="mb-4">
          <CheckCircle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          <span>{t('setup.conflicts_resolved')}</span>
        </StatusCard>
      )}

      {conflictError && (
        <StatusCard type="error" isDark={isDark} className="mb-4">
          <AlertTriangle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          <div>
            <p className="font-medium">{t('setup.conflicts_failed')}</p>
            <p className="text-xs opacity-90">{conflictError}</p>
          </div>
        </StatusCard>
      )}

      {shortcutRegistered && (
        <StatusCard type="success" isDark={isDark} className="mb-4">
          <CheckCircle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          <span>{t('setup.shortcut_success')}</span>
        </StatusCard>
      )}

      {showManualInstructions && shortcutTools && (
        <div className="mb-4 space-y-3">
          <StatusCard type="warning" isDark={isDark}>
            <div>
              <p className="font-medium mb-2">{t('setup.manual_required')}</p>
              <p className="whitespace-pre-line opacity-90 text-xs">
                {t('setup.manual_instructions', {
                  desktop: shortcutTools.desktop_environment,
                  command: 'windows-11-style-clipboard-history-manager',
                })}
              </p>
            </div>
          </StatusCard>
          <WizardButton {...buttonContext} id="copy-path" onClick={onCopyCommand}>
            <span className="flex items-center justify-center gap-2">
              <Copy className="w-4 h-4" />
              {copied ? t('clipboard.copied') : t('setup.copy_command')}
            </span>
          </WizardButton>
        </div>
      )}

      <div className="flex flex-col gap-2 items-center">
        {shortcutTools?.can_register_automatically &&
          !shortcutRegistered &&
          !showManualInstructions && (
            <WizardButton
              {...buttonContext}
              id="register"
              onClick={onRegisterShortcut}
              disabled={registeringShortcut}
              primary
            >
              {registeringShortcut ? t('setup.registering') : t('setup.register_auto')}
            </WizardButton>
          )}

        {!shortcutTools?.can_register_automatically && !showManualInstructions && (
          <WizardButton {...buttonContext} id="show-manual" onClick={onShowManualInstructions}>
            {t('setup.show_manual')}
          </WizardButton>
        )}

        <WizardButton
          {...buttonContext}
          id="shortcut-continue"
          onClick={onContinue}
          primary={shortcutRegistered || showManualInstructions}
        >
          {shortcutRegistered || showManualInstructions ? t('common.continue') : t('common.skip')}
        </WizardButton>
      </div>
    </div>
  )
}
