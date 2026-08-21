import { CheckCircle, Keyboard } from 'lucide-react'
import { clsx } from 'clsx'
import { useTranslation } from 'react-i18next'
import { WizardButton } from './WizardButton'
import type { WizardButtonContext } from './types'

interface DoneStepProps extends WizardButtonContext {
  readonly onFinish: () => void
}

/** Step 4 — completion and the shortcut reminder. / گام ۴ — پایان. */
export function DoneStep({ onFinish, ...buttonContext }: DoneStepProps) {
  const { t } = useTranslation()
  const { isDark } = buttonContext

  return (
    <div className="text-center animate-step-in">
      <div className="mb-6">
        <div
          className={clsx(
            'w-16 h-16 mx-auto rounded-full flex items-center justify-center',
            isDark ? 'bg-win11-success/20' : 'bg-green-100'
          )}
        >
          <CheckCircle className="w-8 h-8 text-win11-success" />
        </div>
      </div>
      <h2
        className={clsx(
          'text-xl font-semibold mb-2',
          isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
        )}
      >
        {t('setup.done_title')}
      </h2>
      <p
        className={clsx(
          'text-sm mb-4',
          isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
        )}
      >
        {t('setup.done_desc')}
      </p>
      <div className="flex items-center justify-center gap-2 mb-6">
        <Keyboard
          className={clsx(
            'w-4 h-4',
            isDark ? 'text-win11-text-tertiary' : 'text-win11Light-text-secondary'
          )}
        />
        <kbd
          className={clsx(
            'px-3 py-1.5 rounded-win11 font-mono text-sm',
            isDark
              ? 'bg-win11-bg-tertiary text-win11-text-primary border border-win11-border-subtle'
              : 'bg-win11Light-bg-tertiary text-win11Light-text-primary border border-win11Light-border'
          )}
        >
          Super + V
        </kbd>
      </div>
      <WizardButton {...buttonContext} id="finish" onClick={onFinish} primary>
        {t('setup.start_using')}
      </WizardButton>
    </div>
  )
}
