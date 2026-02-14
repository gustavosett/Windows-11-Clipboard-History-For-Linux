import { invoke } from '@tauri-apps/api/core'

/** Cached NVIDIA detection result (null = not yet checked) */
let isNvidia: boolean | null = null

/**
 * Check once at startup whether the system has an NVIDIA GPU.
 * Result is cached for subsequent calls.
 */
async function checkIsNvidia(): Promise<boolean> {
  if (isNvidia !== null) return isNvidia
  try {
    isNvidia = await invoke<boolean>('is_nvidia')
  } catch {
    isNvidia = false
  }
  return isNvidia
}

/**
 * Force a window repaint to prevent ghosting on NVIDIA GPUs.
 * This is a no-op on non-NVIDIA systems (skips the IPC call entirely).
 */
export async function forceRepaint(): Promise<void> {
  const nvidia = await checkIsNvidia()
  if (!nvidia) return

  invoke('force_repaint').catch((e) => {
    console.error('Failed to force repaint:', e)
  })
}
