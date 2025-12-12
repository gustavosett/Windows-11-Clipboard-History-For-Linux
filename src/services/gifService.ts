/**
 * GIF Service
 * Handles fetching GIFs from Tenor API v1
 */
import type { Gif } from '../types/gif'

const TENOR_API_KEY = 'LIVDSRZULELA'
const TENOR_API_BASE = 'https://g.tenor.com/v1'
const DEFAULT_LIMIT = 30

interface TenorV1MediaFormat {
  url: string
  dims: [number, number]
  size: number
}

interface TenorV1Media {
  gif?: TenorV1MediaFormat
  mediumgif?: TenorV1MediaFormat
  tinygif?: TenorV1MediaFormat
  nanogif?: TenorV1MediaFormat
}

interface TenorV1Result {
  id: string
  title: string
  media: TenorV1Media[]
  content_description?: string
  itemurl: string
  url: string
  tags: string[]
  created: number
}

interface TenorV1Response {
  results: TenorV1Result[]
  next: string
}

function transformTenorResult(result: TenorV1Result): Gif {
  // v1 API has media as an array, get the first item
  const mediaFormats = result.media[0]
  if (!mediaFormats) {
    throw new Error(`Missing media for GIF: ${result.id}`)
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
 * Fetch trending GIFs from Tenor
 */
export async function fetchTrendingGifs(limit: number = DEFAULT_LIMIT): Promise<Gif[]> {
  const params = new URLSearchParams({
    key: TENOR_API_KEY,
    limit: String(limit),
    media_filter: 'minimal',
  })

  const response = await fetch(`${TENOR_API_BASE}/trending?${params}`)

  if (!response.ok) {
    throw new Error(`Tenor API error: ${response.status} ${response.statusText}`)
  }

  const data: TenorV1Response = await response.json()

  return data.results
    .map((result) => {
      try {
        return transformTenorResult(result)
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
  if (!query.trim()) {
    return fetchTrendingGifs(limit)
  }

  const params = new URLSearchParams({
    key: TENOR_API_KEY,
    q: query.trim(),
    limit: String(limit),
    media_filter: 'minimal',
  })

  const response = await fetch(`${TENOR_API_BASE}/search?${params}`)

  if (!response.ok) {
    throw new Error(`Tenor API error: ${response.status} ${response.statusText}`)
  }

  const data: TenorV1Response = await response.json()

  return data.results
    .map((result) => {
      try {
        return transformTenorResult(result)
      } catch {
        console.warn(`Skipping malformed GIF result: ${result.id}`)
        return null
      }
    })
    .filter((gif): gif is Gif => gif !== null)
}
