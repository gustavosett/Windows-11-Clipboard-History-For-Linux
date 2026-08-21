import type { UserSettings } from '../types/clipboard'

export const DEFAULT_SETTINGS: UserSettings = {
  theme_mode: 'system',
  dark_background_opacity: 0.7,
  light_background_opacity: 0.7,
  language: 'en',
  enable_smart_actions: true,
  enable_ui_polish: true,
  enable_dynamic_tray_icon: true,
  max_history_size: 50,
  auto_delete_interval: 0,
  auto_delete_unit: 'hours',
  custom_kaomojis: [],
  ui_scale: 1,
  filter_secrets: true,
  save_images: true,
  exclude_sensitive_apps: true,
  extra_excluded_apps: [],
  allow_wm_config_rewrite: false,
  history_key_backend: 'file',
}
