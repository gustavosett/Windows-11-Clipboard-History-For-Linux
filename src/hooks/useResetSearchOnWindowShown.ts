import { useEffect, useRef } from 'react'
import { listen } from '@tauri-apps/api/event'

/**
 * Listens for the `window-shown` event and invokes `onReset` when the window is
 * (re)opened, but only while `enabled` is true.
 *
 * Refs keep both `enabled` and `onReset` fresh, so the listener is registered
 * exactly once and callers never need to worry about stale closures.
 */
export function useResetSearchOnWindowShown(enabled: boolean, onReset: () => void) {
  const enabledRef = useRef(enabled)
  const onResetRef = useRef(onReset)

  useEffect(() => {
    enabledRef.current = enabled
    onResetRef.current = onReset
  })

  useEffect(() => {
    const unlisten = listen('window-shown', () => {
      if (enabledRef.current) onResetRef.current()
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])
}
