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
 * Queries the backend once for the rendering environment (NVIDIA / AppImage
 * detection) and caches the result for the lifetime of the React tree.
 *
 * When `transparency_disabled` is `true` the caller should:
 *   - Force opacity to 1.0 (fully opaque)
 *   - Remove rounded outer corners (use `rounded-none`)
 *   - Disable the transparency sliders in Settings
 */
export function useRenderingEnv() {
  const [env, setEnv] = useState<RenderingEnv>(DEFAULT_RENDERING_ENV)

  useEffect(() => {
    invoke<RenderingEnv>('get_rendering_environment')
      .then(setEnv)
      .catch((err) => {
        console.error('Failed to query rendering environment:', err)
      })
  }, [])

  return env
}
