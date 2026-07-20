/**
 * GIF Service
 * Handles fetching GIFs from Klipy API v2 (Tenor compatible)
 */
import { invoke } from '@tauri-apps/api/core'
import type { Gif, KlipyGifResult, KlipySearchResponse } from '../types/gif'
import type { UserSettings } from '../types/clipboard'

const KLIPY_API_BASE = 'https://api.klipy.com/v2'
const DEFAULT_LIMIT = 30
const KLIPY_MEDIA_FILTER = 'gif,tinygif,nanogif,mediumgif'

/**
 * Dynamically retrieves the user's Klipy API key from settings.
 */
async function getKlipyApiKey(): Promise<string> {
  try {
    const settings = await invoke<UserSettings>('get_user_settings')
    return settings.klipy_api_key?.trim() || ''
  } catch (err) {
    console.error('[gifService] Failed to load user settings for API key:', err)
    return ''
  }
}

/**
 * Transforms a Klipy V2 API result into our app's internal Gif structure.
 */
function transformKlipyResult(result: KlipyGifResult): Gif {
  const mediaFormats = result.media_formats
  if (!mediaFormats) {
    throw new Error(`Missing media formats for GIF: ${result.id}`)
  }

  // Use nanogif for preview (smallest size for grid)
  const preview = mediaFormats.nanogif || mediaFormats.tinygif
  // Use tinygif for full display (good balance of quality and size)
  const full = mediaFormats.tinygif || mediaFormats.mediumgif || mediaFormats.gif

  if (!preview || !full) {
    throw new Error(`Missing media formats for GIF: ${result.id}`)
  }

  return {
    id: result.id,
    title: result.content_description || result.title || 'GIF',
    previewUrl: preview.url,
    fullUrl: full.url,
    width: preview.dims[0],
    height: preview.dims[1],
  }
}

/**
 * Fetch trending GIFs from Klipy
 */
export async function fetchTrendingGifs(limit: number = DEFAULT_LIMIT): Promise<Gif[]> {
  const apiKey = await getKlipyApiKey()
  if (!apiKey) {
    throw new Error('Klipy API Key is not configured. Please set it in Settings.')
  }

  const params = new URLSearchParams({
    key: apiKey,
    limit: String(limit),
    media_filter: KLIPY_MEDIA_FILTER,
  })

  const response = await fetch(`${KLIPY_API_BASE}/featured?${params}`)

  if (!response.ok) {
    throw new Error(`Klipy API error: ${response.status} ${response.statusText}`)
  }

  const data: KlipySearchResponse = await response.json()

  return (data.results || [])
    .map((result) => {
      try {
        return transformKlipyResult(result)
      } catch {
        console.warn(`Skipping malformed GIF result: ${result.id}`)
        return null
      }
    })
    .filter((gif): gif is Gif => gif !== null)
}

/**
 * Search GIFs by query
 */
export async function searchGifs(query: string, limit: number = DEFAULT_LIMIT): Promise<Gif[]> {
  const apiKey = await getKlipyApiKey()
  if (!apiKey) {
    throw new Error('Klipy API Key is not configured. Please set it in Settings.')
  }

  if (!query.trim()) {
    return fetchTrendingGifs(limit)
  }

  const params = new URLSearchParams({
    key: apiKey,
    q: query.trim(),
    limit: String(limit),
    media_filter: KLIPY_MEDIA_FILTER,
  })

  const response = await fetch(`${KLIPY_API_BASE}/search?${params}`)

  if (!response.ok) {
    throw new Error(`Klipy API error: ${response.status} ${response.statusText}`)
  }

  const data: KlipySearchResponse = await response.json()

  return (data.results || [])
    .map((result) => {
      try {
        return transformKlipyResult(result)
      } catch {
        console.warn(`Skipping malformed GIF result: ${result.id}`)
        return null
      }
    })
    .filter((gif): gif is Gif => gif !== null)
}
