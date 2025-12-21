import { getCurrentWindow } from '@tauri-apps/api/window'
import { X } from 'lucide-react'
import { clsx } from 'clsx'

interface DragHandleProps {
  isDark: boolean
}

export function DragHandle({ isDark }: DragHandleProps) {
  const appWindow = getCurrentWindow()

  const handleMouseDown = async (e: React.MouseEvent) => {
    if (e.button !== 0) return
    try {
      await appWindow.startDragging()
    } catch (error) {
      console.warn('Window dragging not available:', error)
    }
  }

  const handleClose = (e: React.MouseEvent) => {
    e.stopPropagation()
    appWindow.hide()
  }

  return (
    <div
      data-tauri-drag-region
      className="relative w-full flex justify-center pt-4 pb-1 cursor-grab active:cursor-grabbing select-none"
      onMouseDown={handleMouseDown}
    >
      <div
        data-tauri-drag-region
        className={clsx(
          'w-10 h-1 rounded-full pointer-events-none',
          isDark ? 'bg-white/20' : 'bg-black/20'
        )}
      />

      <button
        onClick={handleClose}
        onMouseDown={(e) => e.stopPropagation()}
        className={clsx(
          'absolute right-4 top-1/2 -translate-y-1/2 p-1 pt-5 rounded-md cursor-pointer z-10',
          isDark ? 'text-white/50' : 'text-black/50'
        )}
        tabIndex={-1}
      >
        <X className="w-5 h-5" />
      </button>
    </div>
  )
}
