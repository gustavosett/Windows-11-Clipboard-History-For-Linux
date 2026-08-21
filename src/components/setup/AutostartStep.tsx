import { Rocket } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { StepHeader } from './StepHeader'
import { WizardButton } from './WizardButton'
import type { WizardButtonContext } from './types'

interface AutostartStepProps extends WizardButtonContext {
  readonly onEnable: () => void
  readonly onSkip: () => void
}

/** Step 3 — optional autostart on login. / گام ۳ — اجرای خودکار هنگام ورود. */
export function AutostartStep({ onEnable, onSkip, ...buttonContext }: AutostartStepProps) {
  const { t } = useTranslation()

  return (
    <div className="animate-step-in">
      <StepHeader
        icon={Rocket}
        title={t('setup.autostart_title')}
        subtitle={t('setup.autostart_desc')}
        isDark={buttonContext.isDark}
      />
      <div className="flex gap-3 justify-center">
        <WizardButton {...buttonContext} id="enable-autostart" onClick={onEnable} primary>
          {t('setup.yes_enable')}
        </WizardButton>
        <WizardButton {...buttonContext} id="skip-autostart" onClick={onSkip}>
          {t('setup.no_thanks')}
        </WizardButton>
      </div>
    </div>
  )
}
