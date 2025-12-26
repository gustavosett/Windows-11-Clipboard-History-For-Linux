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

    // 1. URL Detection
    const urlRegex =
      /^(?!mailto:)(?:(?:http|https|ftp):\/\/)(?:\S+(?::\S*)?@)?(?:(?:(?:[1-9]\d?|1\d\d|2[01]\d|22[0-3])(?:\.(?:1?\d{1,2}|2[0-4]\d|25[0-5])){2}(?:\.(?:[0-9]\d?|1\d\d|2[0-4]\d|25[0-4]))|(?:(?:[a-z\u00a1-\uffff0-9]+-?)*[a-z\u00a1-\uffff0-9]+)(?:\.(?:[a-z\u00a1-\uffff0-9]+-?)*[a-z\u00a1-\uffff0-9]+)*(?:\.(?:[a-z\u00a1-\uffff]{2,})))|localhost)(?::\d{2,5})?(?:(\/|\?|#)(?:[^\s]*[^.\s])?)?$/i

    if (urlRegex.test(trimmed)) {
      const normalizedUrl = trimmed.startsWith('http') ? trimmed : `https://${trimmed}`

      actions.push({
        id: 'open-link',
        label: 'Open Link',
        data: normalizedUrl,
      })
    }

    // 2. Email Detection
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
    if (emailRegex.test(trimmed)) {
      actions.push({ id: 'compose-email', label: 'Compose Email', data: `mailto:${trimmed}` })
    }

    // 3. Color Detection (Hex)
    const hexColorRegex = /^#([0-9A-F]{3}){1,2}$/i
    if (hexColorRegex.test(trimmed)) {
      actions.push({ id: 'color-preview', label: 'Color', data: trimmed })
    }

    // 4. Color Detection (RGB)
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
        // Color preview actions are passive; no additional execution is required
        default:
          console.warn('Unknown smart action', action.id)
      }
    } catch (e) {
      console.error('Failed to execute smart action', e)
      throw e // Propagate error for UI handling
    }
  },
}
