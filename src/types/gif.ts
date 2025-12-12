/**
 * GIF Types
 * TypeScript interfaces for Tenor API responses and GIF data
 */

/** Media format from Tenor API */
export interface TenorMediaFormat {
  url: string
  dims: [number, number]
  duration?: number
  size?: number
}

/** Media formats available for a GIF */
export interface TenorMediaFormats {
  gif?: TenorMediaFormat
  mediumgif?: TenorMediaFormat
  tinygif?: TenorMediaFormat
  nanogif?: TenorMediaFormat
  mp4?: TenorMediaFormat
  loopedmp4?: TenorMediaFormat
  tinymp4?: TenorMediaFormat
  nanomp4?: TenorMediaFormat
  webm?: TenorMediaFormat
  tinywebm?: TenorMediaFormat
  nanowebm?: TenorMediaFormat
}

/** Single GIF result from Tenor API */
export interface TenorGifResult {
  id: string
  title: string
  media_formats: TenorMediaFormats
  content_description: string
  itemurl: string
  url: string
  tags: string[]
  created: number
}

/** Tenor API response for search/trending */
export interface TenorSearchResponse {
  results: TenorGifResult[]
  next: string
}

/** Simplified GIF interface for our app */
export interface Gif {
  id: string
  title: string
  previewUrl: string
  fullUrl: string
  width: number
  height: number
}
