/**
 * GIF Types
 * TypeScript interfaces for Klipy API responses and GIF data
 */

/** Media format from Klipy API */
export interface KlipyMediaFormat {
  url: string
  dims: [number, number]
  duration?: number
  size?: number
}

/** Media formats available for a GIF */
export interface KlipyMediaFormats {
  gif?: KlipyMediaFormat
  mediumgif?: KlipyMediaFormat
  tinygif?: KlipyMediaFormat
  nanogif?: KlipyMediaFormat
  mp4?: KlipyMediaFormat
  loopedmp4?: KlipyMediaFormat
  tinymp4?: KlipyMediaFormat
  nanomp4?: KlipyMediaFormat
  webm?: KlipyMediaFormat
  tinywebm?: KlipyMediaFormat
  nanowebm?: KlipyMediaFormat
}

/** Single GIF result from Klipy API */
export interface KlipyGifResult {
  id: string
  title: string
  media_formats: KlipyMediaFormats
  content_description: string
  itemurl: string
  url: string
  tags: string[]
  created: number
}

/** Klipy API response for search/trending */
export interface KlipySearchResponse {
  results: KlipyGifResult[]
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
