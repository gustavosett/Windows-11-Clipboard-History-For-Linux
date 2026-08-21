import { Rocket } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { StepHeader } from './StepHeader'
import { WizardButton } from './WizardButton'
import type { WizardButtonContext } from './types'

interface WelcomeStepProps extends WizardButtonContext {
  readonly onStart: () => void
}

/** Step 0 — greeting and value proposition. / گام ۰ — خوش‌آمد. */
export function WelcomeStep({ onStart, ...buttonContext }: WelcomeStepProps) {
  const { t } = useTranslation()

  return (
    <div className="text-center animate-step-in">
      <StepHeader
        icon={Rocket}
        title={t('setup.welcome')}
        subtitle={
          <>
            {t('setup.welcome_desc')}
            <br />
            {t('setup.welcome_next')}
          </>
        }
        isDark={buttonContext.isDark}
        large
      />
      <WizardButton {...buttonContext} id="start" onClick={onStart} primary>
        {t('setup.get_started')}
      </WizardButton>
    </div>
  )
}
