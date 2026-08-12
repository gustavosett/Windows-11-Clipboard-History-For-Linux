/**
 * Lazy image content hook.
 *
 * The backend strips image base64 from history payloads to keep IPC small;
 * full image content is fetched on demand via `get_item_content` when an
 * item scrolls into view. Results are cached per item id so switching tabs
 * does not refetch.
 *
 * State is initialized from the per-id cache on mount. The caller is expected
 * to render each item under a stable `key={item.id}` (as HistoryItem does), so
 * an instance never serves a previous item's image and no render-phase state
 * adjustment is needed.
 */
import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { ClipboardItem } from '../types/clipboard'

/** Cache of fetched image base64 keyed by item id. */
const imageContentCache = new Map<string, string>()
/** Set of item ids currently being fetched, to avoid duplicate requests. */
const imageContentInFlight = new Set<string>()

/** Upper bound for the cache; entries beyond this are evicted oldest-first. */
const IMAGE_CACHE_MAX_SIZE = 256

function cacheImageContent(id: string, base64: string) {
  imageContentCache.set(id, base64)
  if (imageContentCache.size > IMAGE_CACHE_MAX_SIZE) {
    const oldest = imageContentCache.keys().next().value
    if (oldest !== undefined) {
      imageContentCache.delete(oldest)
    }
  }
}

export function useLazyImageContent(id: string, shouldLoad: boolean): string {
  const [base64, setBase64] = useState(() => imageContentCache.get(id) ?? '')

  useEffect(() => {
    if (!shouldLoad || base64 || imageContentCache.has(id) || imageContentInFlight.has(id)) {
      return
    }
    imageContentInFlight.add(id)
    invoke<ClipboardItem>('get_item_content', { id })
      .then((item) => {
        if (item && item.content.type === 'Image') {
          cacheImageContent(id, item.content.data.base64)
          setBase64(item.content.data.base64)
        }
      })
      .catch((err) => {
        if (import.meta.env.DEV) {
          console.warn(`Failed to load image content for "${id}":`, err)
        }
      })
      .finally(() => {
        imageContentInFlight.delete(id)
      })
  }, [id, shouldLoad, base64])

  return base64
}
