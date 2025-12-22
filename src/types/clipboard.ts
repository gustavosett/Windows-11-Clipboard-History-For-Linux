/** Clipboard content types */
export type ClipboardContentType = 'text' | 'image'

/** Text content */
export interface TextContent {
  type: 'Text'
  data: string
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
export type ClipboardContent = TextContent | ImageContent

/** A single clipboard history item */
export interface ClipboardItem {
  id: string
  content: ClipboardContent
  timestamp: string
  pinned: boolean
  preview: string
}

/** Active tab in the UI */
export type ActiveTab = 'clipboard' | 'favorites' | 'gifs' | 'emoji' | 'kaomoji'

/** Theme mode */
export type ThemeMode = 'light' | 'dark' | 'system'
