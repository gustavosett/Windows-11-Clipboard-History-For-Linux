import { useMemo, useCallback } from 'react'
import type { MouseEvent } from 'react'
import { smartActionService } from '../services/smartActionService'
import type { ClipboardItem } from '../types/clipboard'
import type { SmartAction } from '../services/smartActionService'

export function useSmartActions(item: ClipboardItem, enableSmartActions: boolean) {
  const smartActions = useMemo(() => {
    if (!enableSmartActions) return []
    if (item.content.type === 'Text') {
      return smartActionService.detectActions(item.content.data)
    }
    return []
  }, [item, enableSmartActions])

  const handleSmartAction = useCallback(async (e: MouseEvent, action: SmartAction) => {
    e.stopPropagation()
    try {
      await smartActionService.execute(action)
    } catch (err) {
      if (import.meta.env.DEV) {
        alert(`Failed to execute action "${action.label}": ${err}`)
      }
    }
  }, [])

  return {
    smartActions,
    colorPreview: smartActions.find((a) => a.id === 'color-preview'),
    linkAction: smartActions.find((a) => a.id === 'open-link'),
    emailAction: smartActions.find((a) => a.id === 'compose-email'),
    handleSmartAction,
  }
}
