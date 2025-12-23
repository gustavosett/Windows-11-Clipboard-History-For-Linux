import { open } from '@tauri-apps/plugin-shell'

export type SmartActionType = 'open-link' | 'compose-email' | 'color-preview'

export interface SmartAction {
  id: SmartActionType
  label: string
  data?: string // extra data like the color hex or the formatted url
}

export const smartActionService = {
  detectActions(content: string): SmartAction[] {
    const actions: SmartAction[] = []
    if (!content) return actions

    const trimmed = content.trim()

    // URL Detection
    // Basic regex for URL, usually sufficient for UI hints
    const urlRegex = /^(https?:\/\/[^\s]+)$/i
    if (urlRegex.test(trimmed)) {
      actions.push({ id: 'open-link', label: 'Open Link', data: trimmed })
    }

    // Email Detection
    const emailRegex = /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/
    if (emailRegex.test(trimmed)) {
      actions.push({ id: 'compose-email', label: 'Compose Email', data: `mailto:${trimmed}` })
    }

    // Color Detection (Hex)
    const hexColorRegex = /^#([0-9A-F]{3}){1,2}$/i
    if (hexColorRegex.test(trimmed)) {
      actions.push({ id: 'color-preview', label: 'Color', data: trimmed })
    }
    // Color Detection (RGB)
    const rgbColorRegex =
      /^rgb\(\s*(25[0-5]|2[0-4]\d|1?\d?\d)\s*,\s*(25[0-5]|2[0-4]\d|1?\d?\d)\s*,\s*(25[0-5]|2[0-4]\d|1?\d?\d)\s*\)$/i
    if (rgbColorRegex.test(trimmed)) {
      actions.push({ id: 'color-preview', label: 'Color', data: trimmed })
    }

    return actions
  },

  async execute(action: SmartAction) {
    try {
      switch (action.id) {
        case 'open-link':
          if (action.data) await open(action.data)
          break
        case 'compose-email':
          if (action.data) await open(action.data)
          break
        // Color preview is passive, no execution needed usually,
        // but could open color picker or something in future
        default:
          console.warn('Unknown smart action', action.id)
      }
    } catch (e) {
      console.error('Failed to execute smart action', e)
      throw e // Propagate error for UI handling
    }
  },
}
