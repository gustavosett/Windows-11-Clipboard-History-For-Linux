/**
 * GIF Service
 * Fetches GIFs from the backend Tenor API proxy (key lives server-side).
 * No API key is exposed in the frontend bundle.
 */
import { invoke } from '@tauri-apps/api/core'
import type { Gif } from '../types/gif'

const DEFAULT_LIMIT = 30

/**
 * Transform backend GifResult to frontend Gif type
 */
function transformGifResult(r: {
  id: string
  title: string
  preview_url: string
  full_url: string
  width: number
  height: number
}): Gif {
  return {
    id: r.id,
    title: r.title || 'GIF',
    previewUrl: r.preview_url,
    fullUrl: r.full_url,
    width: r.width,
    height: r.height,
  }
}

/**
 * Fetch trending GIFs via backend proxy
 */
export async function fetchTrendingGifs(limit: number = DEFAULT_LIMIT): Promise<Gif[]> {
  try {
    const results = await invoke<GifResult[]>('search_tenor', { query: null, limit })
    return results.map(transformGifResult)
  } catch (err) {
    console.error('[GifService] Failed to fetch trending GIFs:', err)
    return []
  }
}

/**
 * Search GIFs by query via backend proxy
 */
export async function searchGifs(query: string, limit: number = DEFAULT_LIMIT): Promise<Gif[]> {
  if (!query.trim()) {
    return fetchTrendingGifs(limit)
  }

  try {
    const results = await invoke<GifResult[]>('search_tenor', { query: query.trim(), limit })
    return results.map(transformGifResult)
  } catch (err) {
    console.error('[GifService] Failed to search GIFs:', err)
    return []
  }
}

// Backend response type (mirrors Rust GifResult)
interface GifResult {
  id: string
  title: string
  preview_url: string
  full_url: string
  width: number
  height: number
}
