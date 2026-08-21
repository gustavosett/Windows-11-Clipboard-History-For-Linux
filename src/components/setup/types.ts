/**
 * Shared DTO types for the first-run setup wizard.
 * These mirror the Rust command payloads (`permission_checker`,
 * `shortcut_setup`) one-to-one; keep them in sync with the backend structs.
 *
 * انواع مشترک جادوگر راه‌اندازی اولیه؛ بازتاب یک‌به‌یک فرمان‌های Rust
 * هستند و باید با ساختارهای بک‌اند همگام بمانند.
 */

export interface PermissionStatus {
  readonly uinput_accessible: boolean
  readonly uinput_path: string
  readonly user_in_input_group: boolean
  readonly status_code: 'permissions_ok' | 'relogin_required' | 'permissions_missing'
}

export interface ShortcutToolsStatus {
  readonly desktop_environment: string
  readonly gsettings_available: boolean
  readonly kde_tools_available: boolean
  readonly xfce_tools_available: boolean
  readonly can_register_automatically: boolean
  readonly has_conflicts: boolean
  readonly conflict_count: number
  readonly can_auto_resolve_conflicts: boolean
}

export interface ShortcutConflict {
  readonly binding: string
  readonly current_action: string
  readonly owner: string
  readonly resolution_command: string | null
  readonly resolution_steps: string
}

export interface ConflictDetectionResult {
  readonly desktop_environment: string
  readonly conflicts: readonly ShortcutConflict[]
  readonly can_auto_resolve: boolean
  readonly message: string
}

/** Shared visual context the wizard buttons need from their host step. */
/** زمینهٔ بصری مشترکی که دکمه‌های جادوگر از گام میزبان نیاز دارند. */
export interface WizardButtonContext {
  readonly hoveredButton: string | null
  readonly setHoveredButton: (id: string | null) => void
  readonly isDark: boolean
  readonly tertiaryOpacity: number
}
