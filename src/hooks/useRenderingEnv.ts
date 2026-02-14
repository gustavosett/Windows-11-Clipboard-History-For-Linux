import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { RenderingEnv } from '../types/clipboard'

const DEFAULT_RENDERING_ENV: RenderingEnv = {
  is_nvidia: false,
  is_appimage: false,
  transparency_disabled: false,
  reason: '',
}

/**
 * Cached rendering environment shared across all hook consumers.
 * This avoids repeated IPC calls to `get_rendering_environment`.
 */
let cachedEnv: RenderingEnv | null = null

/**
 * Tracks an in-flight request so concurrent hook calls can share the same
 * promise instead of issuing multiple IPC calls.
 */
let pendingEnvPromise: Promise<RenderingEnv> | null = null

/**
 * Queries the backend once for the rendering environment (NVIDIA / AppImage
 * detection) and caches the result process-wide.
 *
 * Multiple components can safely call this hook — only one IPC call will be made.
 *
 * When `transparency_disabled` is `true` the caller should:
 *   - Force opacity to 1 (fully opaque)
 *   - Remove rounded outer corners (use `rounded-none`)
 *   - Disable the transparency sliders in Settings
 */
export function useRenderingEnv() {
  const [env, setEnv] = useState<RenderingEnv>(cachedEnv ?? DEFAULT_RENDERING_ENV)

  useEffect(() => {
    // If we already have a cached env, state is already initialized - skip IPC
    if (cachedEnv) {
      return
    }

    // Reuse an in-flight request if one exists
    if (!pendingEnvPromise) {
      pendingEnvPromise = invoke<RenderingEnv>('get_rendering_environment')
        .then((result) => {
          cachedEnv = result
          return result
        })
        .catch((err) => {
          console.error('Failed to query rendering environment:', err)
          // On error, keep cachedEnv as-is (likely null) so we can retry
          // on next mount if desired
          throw err
        })
        .finally(() => {
          pendingEnvPromise = null
        })
    }

    pendingEnvPromise
      .then((result) => {
        // Guard against cases where the component unmounts; React ignores
        // state updates on unmounted components so this is safe
        setEnv(result)
      })
      .catch(() => {
        // Error already logged above; keep default env in state
      })
  }, [])

  return env
}
