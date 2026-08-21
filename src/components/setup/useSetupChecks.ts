import { useCallback, useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { ConflictDetectionResult, PermissionStatus, ShortcutToolsStatus } from './types'

/**
 * Runs the wizard's three backend probes (uinput permissions, DE shortcut
 * tools, shortcut conflicts) once on mount and exposes their results plus
 * targeted refreshers. Failures are logged and leave the previous state —
 * the UI degrades to "skip" instead of blocking setup.
 * / سه بررسی بک‌اند (مجوز uinput، ابزار میانبر میزکار، تداخل میانبر) را
 * یک‌بار در mount اجرا می‌کند و به‌همراه تازه‌سازهای هدفمند ارائه می‌دهد.
 */
export function useSetupChecks() {
  const [permissions, setPermissions] = useState<PermissionStatus | null>(null)
  const [shortcutTools, setShortcutTools] = useState<ShortcutToolsStatus | null>(null)
  const [conflicts, setConflicts] = useState<ConflictDetectionResult | null>(null)

  const checkPermissions = useCallback(async () => {
    try {
      const status = await invoke<PermissionStatus>('check_permissions')
      setPermissions(status)
    } catch (e) {
      console.error('Failed to check permissions:', e)
    }
  }, [])

  const checkShortcutTools = useCallback(async () => {
    try {
      const status = await invoke<ShortcutToolsStatus>('check_shortcut_tools')
      setShortcutTools(status)
    } catch (e) {
      console.error('Failed to check shortcut tools:', e)
    }
  }, [])

  const checkConflicts = useCallback(async () => {
    try {
      const result = await invoke<ConflictDetectionResult>('detect_conflicts')
      setConflicts(result)
    } catch (e) {
      console.error('Failed to check conflicts:', e)
    }
  }, [])

  useEffect(() => {
    const initialChecksTimer = globalThis.setTimeout(() => {
      void checkPermissions()
      void checkShortcutTools()
      void checkConflicts()
    }, 0)

    return () => globalThis.clearTimeout(initialChecksTimer)
  }, [checkPermissions, checkShortcutTools, checkConflicts])

  return {
    permissions,
    shortcutTools,
    conflicts,
    checkPermissions,
    checkShortcutTools,
    checkConflicts,
  } as const
}
