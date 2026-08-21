/**
 * Setup wizard building blocks. The orchestrator lives in
 * `../SetupWizard.tsx`; each step and shared visual is a single-purpose
 * component so steps stay independently testable and readable.
 * / اجزای جادوگر راه‌اندازی؛ هماهنگ‌کننده در `../SetupWizard.tsx` است و
 * هر گام/عنصر مشترک مؤلفه‌ای مستقل و قابل‌تست است.
 */
export { SetupWizard } from '../SetupWizard'
export { WizardButton } from './WizardButton'
export { StatusCard, type StatusType } from './StatusCard'
export { StepHeader } from './StepHeader'
export { LanguageSwitcher } from './LanguageSwitcher'
export { useSetupChecks } from './useSetupChecks'
export type {
  PermissionStatus,
  ShortcutToolsStatus,
  ShortcutConflict,
  ConflictDetectionResult,
  WizardButtonContext,
} from './types'
