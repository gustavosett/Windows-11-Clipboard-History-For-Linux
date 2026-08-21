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
// export type ActiveTab = 'clipboard' | 'gifs' | 'emoji' | 'kaomoji' | 'symbols'
export type ActiveTab = 'clipboard' | 'emoji' | 'kaomoji' | 'symbols'

/** Theme mode */
export type ThemeMode = 'light' | 'dark' | 'system'

/** Color scheme from XDG Desktop Portal */
export type ColorScheme = 'nopreference' | 'dark' | 'light'

/** System theme information from the backend */
export interface ThemeInfo {
  color_scheme: ColorScheme
  prefers_dark: boolean
  source: string
}

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
  language: string
  enable_smart_actions: boolean
  enable_ui_polish: boolean
  enable_dynamic_tray_icon: boolean
  max_history_size: number
  auto_delete_interval: number
  auto_delete_unit: 'minutes' | 'hours' | 'days' | 'weeks'
  custom_kaomojis: CustomKaomoji[]
  ui_scale: number
  filter_secrets: boolean
  save_images: boolean
  exclude_sensitive_apps: boolean
  extra_excluded_apps: string[]
  allow_wm_config_rewrite: boolean
  /** Where the history encryption key lives ("file" | "secret-service").
   *  محل ذخیرهٔ کلید رمزنگاری تاریخچه ("file" | "secret-service"). */
  history_key_backend: 'file' | 'secret-service'
}

/** Helper type for boolean settings keys */
export type BooleanSettingKey = {
  [K in keyof UserSettings]: UserSettings[K] extends boolean ? K : never
}[keyof UserSettings]

/** Bounded history window returned by `get_history_page` (ADR-0007).
 *  پنجرهٔ محدود تاریخچه که `get_history_page` برمی‌گرداند (ADR-0007). */
export interface HistoryPage {
  items: ClipboardItem[]
  total: number
  limit: number
  offset: number
}

/** Snapshot of the encryption-key backend for the Settings UI (ADR-0006).
 *  وضعیت لحظه‌ای بک‌اند کلید رمزنگاری برای رابط تنظیمات (ADR-0006). */
export interface KeyBackendStatus {
  /** Backend requested by the persisted user setting. */
  setting: 'file' | 'secret-service'
  /** Backend actually in use by this process. */
  active: 'file' | 'secret-service'
  /** True when `secret-tool` (Secret Service) is usable on this machine. */
  secret_service_available: boolean
  /** True when the active backend differs from the setting (restart needed). */
  restart_required: boolean
}

/** Rendering environment flags from the backend (NVIDIA / AppImage detection) */
export interface RenderingEnv {
  /** true when an NVIDIA GPU is detected */
  is_nvidia: boolean
  /** true when running from an AppImage */
  is_appimage: boolean
  /** true when transparency & rounded corners must be disabled */
  transparency_disabled: boolean
  /** Human-readable reason shown in Settings UI */
  reason: string
}
