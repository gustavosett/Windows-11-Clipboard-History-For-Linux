import { clsx } from 'clsx'
import { ExternalLink, Mail } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { SmartAction } from '../services/smartActionService'

export function HistorySmartActions({
  linkAction,
  emailAction,
  isDark,
  onActionClick,
}: {
  linkAction?: SmartAction
  emailAction?: SmartAction
  isDark: boolean
  onActionClick: (e: React.MouseEvent, action: SmartAction) => void
}) {
  const { t } = useTranslation()
  if (!linkAction && !emailAction) return null

  const buttonClasses = (isDark: boolean) =>
    clsx(
      'p-1.5 rounded-md transition-colors',
      isDark
        ? 'text-win11-text-tertiary hover:bg-win11-bg-tertiary'
        : 'text-win11Light-text-secondary hover:bg-win11Light-bg-tertiary'
    )

  return (
    <>
      {linkAction && (
        <button
          onClick={(e) => onActionClick(e, linkAction)}
          aria-label={t('smart_actions.open_link')}
          className={buttonClasses(isDark)}
          title={t('smart_actions.open_link')}
          tabIndex={-1}
        >
          <ExternalLink className="w-4 h-4" />
        </button>
      )}
      {emailAction && (
        <button
          onClick={(e) => onActionClick(e, emailAction)}
          aria-label={t('smart_actions.compose_email')}
          className={buttonClasses(isDark)}
          title={t('smart_actions.compose_email')}
          tabIndex={-1}
        >
          <Mail className="w-4 h-4" />
        </button>
      )}
    </>
  )
}
