/** Clipboard content types */
export type ClipboardContentType = 'text' | 'RichText' | 'image'

/** Text content */
export interface TextContent {
  type: 'Text'
  data: string
}

/** Rich text content with HTML formatting */
export interface RichTextContent {
  type: 'RichText'
  data: {
    plain: string
    html: string
  }
}

/** Image content */
export interface ImageContent {
  type: 'Image'
  data: {
    base64: string
    width: number
    height: number
  }
}

/** Union of all content types */
export type ClipboardContent = TextContent | RichTextContent | ImageContent

/** A single clipboard history item */
export interface ClipboardItem {
  id: string
  content: ClipboardContent
  timestamp: string
  pinned: boolean
  preview: string
}

/** Active tab in the UI */
export type ActiveTab = 'clipboard' | 'gifs' | 'emoji' | 'kaomoji' | 'symbols'

/** Theme mode */
export type ThemeMode = 'light' | 'dark' | 'system'

export interface Kaomoji {
  id: string
  text: string
  category: string
  keywords: string[]
}

export type CustomKaomoji = Omit<Kaomoji, 'id'>

export interface UserSettings {
  theme_mode: ThemeMode
  dark_background_opacity: number
  light_background_opacity: number

  enable_ui_polish: boolean
  custom_kaomojis: CustomKaomoji[]
}
