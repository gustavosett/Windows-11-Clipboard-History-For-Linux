import { clsx } from 'clsx'
import { getTertiaryBackgroundStyle } from '../../utils/themeUtils'
import type { WizardButtonContext } from './types'

interface WizardButtonProps extends WizardButtonContext {
  readonly id: string
  readonly onClick: () => void
  readonly children: React.ReactNode
  readonly disabled?: boolean
  readonly primary?: boolean
}

/**
 * A single wizard button with the Windows 11 hover treatment:
 * tertiary surface on hover, accent text when primary, springy press.
 * / دکمهٔ جادوگر با رفتار هاور ویندوز ۱۱.
 */
export function WizardButton({
  id,
  onClick,
  children,
  disabled = false,
  primary = false,
  hoveredButton,
  setHoveredButton,
  isDark,
  tertiaryOpacity,
}: WizardButtonProps) {
  const isHovered = hoveredButton === id

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      onMouseEnter={() => setHoveredButton(id)}
      onMouseLeave={() => setHoveredButton(null)}
      className={clsx(
        'px-5 py-2.5 rounded-win11 font-medium transition-all duration-150',
        'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent',
        'disabled:opacity-50 disabled:cursor-not-allowed',
        'active:scale-[0.98]',
        primary
          ? 'text-win11-bg-accent'
          : isDark
            ? 'text-win11-text-secondary'
            : 'text-win11Light-text-secondary'
      )}
      style={
        isHovered && !disabled ? getTertiaryBackgroundStyle(isDark, tertiaryOpacity) : undefined
      }
    >
      {children}
    </button>
  )
}
