import { AlertTriangle, CheckCircle, Shield } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { StepHeader } from './StepHeader'
import { StatusCard } from './StatusCard'
import { WizardButton } from './WizardButton'
import type { PermissionStatus, WizardButtonContext } from './types'

interface PermissionsStepProps extends WizardButtonContext {
  readonly permissions: PermissionStatus | null
  readonly fixError: string | null
  readonly fixing: boolean
  readonly onFix: () => void
  readonly onContinue: () => void
}

/**
 * Step 1 — /dev/uinput access check with one-click ACL repair through
 * `pkexec setfacl` (handled by the host).
 * / گام ۱ — بررسی دسترسی /dev/uinput با تعمیر یک‌کلیکی ACL.
 */
export function PermissionsStep({
  permissions,
  fixError,
  fixing,
  onFix,
  onContinue,
  ...buttonContext
}: PermissionsStepProps) {
  const { t } = useTranslation()
  const { isDark } = buttonContext

  return (
    <div className="animate-step-in">
      <StepHeader
        icon={Shield}
        title={t('setup.step_permissions')}
        subtitle={t('setup.permission_required')}
        isDark={isDark}
      />

      {permissions && (
        <StatusCard
          type={permissions.uinput_accessible ? 'success' : 'warning'}
          isDark={isDark}
          className="mb-4"
        >
          {permissions.uinput_accessible ? (
            <CheckCircle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          ) : (
            <AlertTriangle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          )}
          <span>{t(`setup.${permissions.status_code}`)}</span>
        </StatusCard>
      )}

      {fixError && (
        <StatusCard type="error" isDark={isDark} className="mb-4">
          {fixError}
        </StatusCard>
      )}

      <div className="flex gap-3 justify-center">
        {!permissions?.uinput_accessible && (
          <WizardButton
            {...buttonContext}
            id="fix"
            onClick={onFix}
            disabled={fixing}
          >
            {fixing ? t('setup.fixing') : t('setup.fix_now')}
          </WizardButton>
        )}
        <WizardButton {...buttonContext} id="perm-continue" onClick={onContinue} primary>
          {permissions?.uinput_accessible ? t('common.continue') : t('common.skip')}
        </WizardButton>
      </div>
    </div>
  )
}
