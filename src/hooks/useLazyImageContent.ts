/**
 * Lazy image content hook.
 *
 * The backend strips image base64 from history payloads to keep IPC small;
 * full image content is fetched on demand via `get_item_content` when an
 * item scrolls into view. Results are cached per item id so switching tabs
 * does not refetch.
 */
import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { ClipboardItem } from '../types/clipboard'

/** Cache of fetched image base64 keyed by item id. */
const imageContentCache = new Map<string, string>()
/** Set of item ids currently being fetched, to avoid duplicate requests. */
const imageContentInFlight = new Set<string>()

export function useLazyImageContent(id: string, shouldLoad: boolean): string {
  const [base64, setBase64] = useState(() => imageContentCache.get(id) ?? '')

  // Adjust state when the item id changes (e.g. virtualized row reuse) so a
  // reused instance never shows a previous item's image.
  const [prevId, setPrevId] = useState(id)
  if (prevId !== id) {
    setPrevId(id)
    setBase64(imageContentCache.get(id) ?? '')
  }

  useEffect(() => {
    if (!shouldLoad || base64 || imageContentCache.has(id) || imageContentInFlight.has(id)) {
      return
    }
    imageContentInFlight.add(id)
    invoke<ClipboardItem>('get_item_content', { id })
      .then((item) => {
        if (item && item.content.type === 'Image') {
          imageContentCache.set(id, item.content.data.base64)
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
