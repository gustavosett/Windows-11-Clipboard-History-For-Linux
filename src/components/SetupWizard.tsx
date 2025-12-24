import { invoke } from '@tauri-apps/api/core'
import { useState, useEffect } from 'react'
import { clsx } from 'clsx'
import { useAutostart } from '../hooks/useAutostart'
import { getTertiaryBackgroundStyle } from '../utils/themeUtils'
import {
  CheckCircle,
  AlertTriangle,
  Shield,
  Rocket,
  Keyboard,
  Settings,
  Copy,
  AlertCircle,
  Zap,
} from 'lucide-react'

interface PermissionStatus {
  uinput_accessible: boolean
  uinput_path: string
  user_in_input_group: boolean
  suggestion: string
}

interface ShortcutToolsStatus {
  desktop_environment: string
  gsettings_available: boolean
  kde_tools_available: boolean
  xfce_tools_available: boolean
  can_register_automatically: boolean
  manual_instructions: string
  has_conflicts: boolean
  conflict_count: number
  can_auto_resolve_conflicts: boolean
}

interface ShortcutConflict {
  binding: string
  current_action: string
  owner: string
  resolution_command: string | null
  resolution_steps: string
}

interface ConflictDetectionResult {
  desktop_environment: string
  conflicts: ShortcutConflict[]
  can_auto_resolve: boolean
  message: string
}

interface SetupWizardProps {
  readonly onComplete: () => void
}

function useSystemDarkMode(): boolean {
  const [isDark, setIsDark] = useState(() => {
    if (globalThis.matchMedia) {
      return globalThis.matchMedia('(prefers-color-scheme: dark)').matches
    }
    return true
  })

  useEffect(() => {
    const mediaQuery = globalThis.matchMedia('(prefers-color-scheme: dark)')
    const handleChange = (e: MediaQueryListEvent) => setIsDark(e.matches)
    mediaQuery.addEventListener('change', handleChange)
    return () => mediaQuery.removeEventListener('change', handleChange)
  }, [])

  return isDark
}

interface WizardButtonProps {
  id: string
  onClick: () => void
  children: React.ReactNode
  disabled?: boolean
  primary?: boolean
  hoveredButton: string | null
  setHoveredButton: (id: string | null) => void
  isDark: boolean
  tertiaryOpacity: number
}

const WizardButton = ({
  id,
  onClick,
  children,
  disabled = false,
  primary = false,
  hoveredButton,
  setHoveredButton,
  isDark,
  tertiaryOpacity,
}: WizardButtonProps) => {
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

export function SetupWizard({ onComplete }: SetupWizardProps) {
  const [step, setStep] = useState(0)
  const [permissions, setPermissions] = useState<PermissionStatus | null>(null)
  const [shortcutTools, setShortcutTools] = useState<ShortcutToolsStatus | null>(null)
  const [conflicts, setConflicts] = useState<ConflictDetectionResult | null>(null)
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
  const isDark = useSystemDarkMode()

  // Fixed opacity for the wizard (similar to main app default)
  const tertiaryOpacity = 0.85

  // Local wrapper to reduce prop drilling on every WizardButton usage
  const Button = (
    props: Omit<
      WizardButtonProps,
      'hoveredButton' | 'setHoveredButton' | 'isDark' | 'tertiaryOpacity'
    >
  ) => (
    <WizardButton
      {...props}
      hoveredButton={hoveredButton}
      setHoveredButton={setHoveredButton}
      isDark={isDark}
      tertiaryOpacity={tertiaryOpacity}
    />
  )

  useEffect(() => {
    if (isDark) {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }, [isDark])

  useEffect(() => {
    checkPermissions()
    checkShortcutTools()
    checkConflicts()
  }, [])

  const checkPermissions = async () => {
    try {
      const status = await invoke<PermissionStatus>('check_permissions')
      setPermissions(status)
    } catch (e) {
      console.error('Failed to check permissions:', e)
    }
  }

  const checkShortcutTools = async () => {
    try {
      const status = await invoke<ShortcutToolsStatus>('check_shortcut_tools')
      setShortcutTools(status)
    } catch (e) {
      console.error('Failed to check shortcut tools:', e)
    }
  }

  const checkConflicts = async () => {
    try {
      const result = await invoke<ConflictDetectionResult>('detect_conflicts')
      setConflicts(result)
    } catch (e) {
      console.error('Failed to check conflicts:', e)
    }
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
      setConflictError(String(e))
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
      setFixError(String(e))
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

  const handleComplete = async () => {
    try {
      await invoke('mark_first_run_complete')
    } catch (e) {
      console.error('Failed to mark first run complete:', e)
    }
    onComplete()
  }

  const copyToClipboard = (text: string) => {
    navigator.clipboard.writeText(text)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  // Status message styles
  const statusCardClass = (type: 'success' | 'warning' | 'error') =>
    clsx(
      'p-4 rounded-win11 flex items-start gap-3 text-sm',
      type === 'success' &&
        (isDark
          ? 'bg-win11-success/15 text-win11-success border border-win11-success/20'
          : 'bg-green-50 text-green-700 border border-green-200'),
      type === 'warning' &&
        (isDark
          ? 'bg-win11-warning/15 text-win11-warning border border-win11-warning/20'
          : 'bg-amber-50 text-amber-700 border border-amber-200'),
      type === 'error' &&
        (isDark
          ? 'bg-win11-error/15 text-win11-error border border-win11-error/20'
          : 'bg-red-50 text-red-700 border border-red-200')
    )

  const infoCardClass = clsx(
    'p-3 rounded-win11',
    isDark
      ? 'bg-win11-bg-tertiary/50 border border-win11-border-subtle'
      : 'bg-win11Light-bg-tertiary/50 border border-win11Light-border'
  )

  const steps = [
    // Step 0: Welcome
    <div key="welcome" className="text-center animate-fade-in">
      <div className="mb-6">
        <div
          className={clsx(
            'w-16 h-16 mx-auto rounded-full flex items-center justify-center',
            isDark ? 'bg-win11-bg-tertiary' : 'bg-win11Light-bg-tertiary'
          )}
        >
          <Rocket
            className={clsx(
              'w-8 h-8',
              isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
            )}
          />
        </div>
      </div>
      <h2
        className={clsx(
          'text-xl font-semibold mb-2',
          isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
        )}
      >
        Welcome to Clipboard History
      </h2>
      <p
        className={clsx(
          'text-sm mb-8',
          isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
        )}
      >
        A Windows 11-style clipboard manager for Linux.
        <br />
        Let's set up a few things to get you started.
      </p>
      <Button id="start" onClick={() => setStep(1)} primary>
        Get Started
      </Button>
    </div>,

    // Step 1: Permissions
    <div key="permissions" className="animate-fade-in">
      <div className="text-center mb-6">
        <div
          className={clsx(
            'w-14 h-14 mx-auto rounded-full flex items-center justify-center mb-4',
            isDark ? 'bg-win11-bg-tertiary' : 'bg-win11Light-bg-tertiary'
          )}
        >
          <Shield
            className={clsx(
              'w-7 h-7',
              isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
            )}
          />
        </div>
        <h2
          className={clsx(
            'text-lg font-semibold mb-1',
            isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
          )}
        >
          Input Permissions
        </h2>
        <p
          className={clsx(
            'text-sm',
            isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
          )}
        >
          Required to simulate Ctrl+V for pasting.
        </p>
      </div>

      {permissions && (
        <div
          className={clsx(
            'mb-4',
            statusCardClass(permissions.uinput_accessible ? 'success' : 'warning')
          )}
        >
          {permissions.uinput_accessible ? (
            <CheckCircle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          ) : (
            <AlertTriangle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          )}
          <span>{permissions.suggestion}</span>
        </div>
      )}

      {fixError && <div className={clsx('mb-4', statusCardClass('error'))}>{fixError}</div>}

      <div className="flex gap-3 justify-center">
        {!permissions?.uinput_accessible && (
          <Button id="fix" onClick={handleFixPermissions} disabled={fixing}>
            {fixing ? 'Fixing...' : 'Fix Now'}
          </Button>
        )}
        <Button id="perm-continue" onClick={() => setStep(2)} primary>
          {permissions?.uinput_accessible ? 'Continue' : 'Skip'}
        </Button>
      </div>
    </div>,

    // Step 2: Shortcut Configuration
    <div key="shortcut" className="animate-fade-in">
      <div className="text-center mb-6">
        <div
          className={clsx(
            'w-14 h-14 mx-auto rounded-full flex items-center justify-center mb-4',
            isDark ? 'bg-win11-bg-tertiary' : 'bg-win11Light-bg-tertiary'
          )}
        >
          <Keyboard
            className={clsx(
              'w-7 h-7',
              isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
            )}
          />
        </div>
        <h2
          className={clsx(
            'text-lg font-semibold mb-1',
            isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
          )}
        >
          Keyboard Shortcut
        </h2>
        <p
          className={clsx(
            'text-sm',
            isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
          )}
        >
          Set up{' '}
          <kbd
            className={clsx(
              'px-2 py-0.5 rounded text-xs font-mono',
              isDark ? 'bg-win11-bg-tertiary' : 'bg-win11Light-bg-tertiary'
            )}
          >
            Super + V
          </kbd>{' '}
          to open clipboard.
        </p>
      </div>

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
              Detected:{' '}
              <strong
                className={isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'}
              >
                {shortcutTools.desktop_environment}
              </strong>
            </span>
          </div>
        </div>
      )}

      {/* Conflict Warning */}
      {conflicts && conflicts.conflicts.length > 0 && !conflictsResolved && (
        <div className={clsx('mb-4', statusCardClass('warning'))}>
          <AlertCircle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          <div className="flex-1">
            <p className="font-medium mb-1">
              {conflicts.conflicts.length} shortcut conflict
              {conflicts.conflicts.length > 1 ? 's' : ''} detected
            </p>
            <p className="text-xs opacity-90 mb-2">
              Super+V is already used by {conflicts.conflicts[0].owner} for "
              {conflicts.conflicts[0].current_action}"
            </p>
            {conflicts.can_auto_resolve && (
              <div className="space-y-1">
                <Button
                  id="resolve-conflicts"
                  onClick={handleResolveConflicts}
                  disabled={resolvingConflicts}
                >
                  <span className="flex items-center gap-2">
                    <Zap className="w-4 h-4" />
                    {resolvingConflicts ? 'Resolving...' : 'Auto-Fix Conflicts'}
                  </span>
                </Button>
                <p className="text-xs opacity-60">
                  This will comment out conflicting lines in your config. A backup will be created.
                </p>
              </div>
            )}
            {!conflicts.can_auto_resolve && (
              <p className="text-xs opacity-75 mt-1">
                Manual resolution required. See instructions below.
              </p>
            )}
          </div>
        </div>
      )}

      {conflictsResolved && (
        <div className={clsx('mb-4', statusCardClass('success'))}>
          <CheckCircle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          <span>Conflicts resolved! Super+V is now available.</span>
        </div>
      )}

      {conflictError && (
        <div className={clsx('mb-4', statusCardClass('error'))}>
          <AlertTriangle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          <div>
            <p className="font-medium">Failed to resolve conflicts</p>
            <p className="text-xs opacity-90">{conflictError}</p>
          </div>
        </div>
      )}

      {shortcutRegistered && (
        <div className={clsx('mb-4', statusCardClass('success'))}>
          <CheckCircle className="w-5 h-5 flex-shrink-0 mt-0.5" />
          <span>Shortcut registered successfully!</span>
        </div>
      )}

      {showManualInstructions && shortcutTools && (
        <div className="mb-4 space-y-3">
          <div className={statusCardClass('warning')}>
            <div>
              <p className="font-medium mb-2">Manual Setup Required:</p>
              <p className="whitespace-pre-line opacity-90 text-xs">
                {shortcutTools.manual_instructions}
              </p>
            </div>
          </div>
          <Button
            id="copy-path"
            onClick={() => copyToClipboard('/usr/bin/win11-clipboard-history')}
          >
            <span className="flex items-center justify-center gap-2">
              <Copy className="w-4 h-4" />
              {copied ? 'Copied!' : 'Copy command path'}
            </span>
          </Button>
        </div>
      )}

      <div className="flex flex-col gap-2 items-center">
        {shortcutTools?.can_register_automatically &&
          !shortcutRegistered &&
          !showManualInstructions && (
            <Button
              id="register"
              onClick={handleRegisterShortcut}
              disabled={registeringShortcut}
              primary
            >
              {registeringShortcut ? 'Registering...' : 'Register Automatically'}
            </Button>
          )}

        {!shortcutTools?.can_register_automatically && !showManualInstructions && (
          <Button id="show-manual" onClick={() => setShowManualInstructions(true)}>
            Show Manual Instructions
          </Button>
        )}

        <Button
          id="shortcut-continue"
          onClick={() => setStep(3)}
          primary={shortcutRegistered || showManualInstructions}
        >
          {shortcutRegistered || showManualInstructions ? 'Continue' : 'Skip'}
        </Button>
      </div>
    </div>,

    // Step 3: Autostart
    <div key="autostart" className="animate-fade-in">
      <div className="text-center mb-6">
        <div
          className={clsx(
            'w-14 h-14 mx-auto rounded-full flex items-center justify-center mb-4',
            isDark ? 'bg-win11-bg-tertiary' : 'bg-win11Light-bg-tertiary'
          )}
        >
          <Rocket
            className={clsx(
              'w-7 h-7',
              isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
            )}
          />
        </div>
        <h2
          className={clsx(
            'text-lg font-semibold mb-1',
            isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
          )}
        >
          Start on Login?
        </h2>
        <p
          className={clsx(
            'text-sm',
            isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
          )}
        >
          Start automatically when you log in?
        </p>
      </div>

      <div className="flex gap-3 justify-center">
        <Button id="enable-autostart" onClick={handleEnableAutostart} primary>
          Yes, enable
        </Button>
        <Button id="skip-autostart" onClick={() => setStep(4)}>
          No thanks
        </Button>
      </div>
    </div>,

    // Step 4: Done
    <div key="done" className="text-center animate-fade-in">
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
        You're all set!
      </h2>
      <p
        className={clsx(
          'text-sm mb-4',
          isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
        )}
      >
        Press the shortcut anytime to open clipboard history.
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
      <Button id="finish" onClick={handleComplete} primary>
        Start Using
      </Button>
    </div>,
  ]

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
    <div className="fixed inset-0 flex items-center justify-center z-50 p-4">
      {/* Backdrop */}
      <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" />

      {/* Modal */}
      <div
        className={clsx(
          'relative w-full max-w-sm rounded-win11-lg p-6 shadow-win11-elevated animate-scale-in',
          isDark ? 'glass-effect' : 'glass-effect-light',
          isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
        )}
      >
        {steps[step]}

        {/* Progress dots */}
        <div className="flex justify-center gap-2 mt-6">
          {steps.map((_, i) => (
            <button
              key={`dot-${i}`}
              onClick={() => i < step && setStep(i)}
              disabled={i >= step}
              aria-label={`Step ${i + 1}`}
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
