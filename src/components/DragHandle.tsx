import { getCurrentWindow } from '@tauri-apps/api/window'
import { X } from 'lucide-react'

export function DragHandle() {
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
        className="w-10 h-1 rounded-full dark:bg-white/20 bg-black/20 pointer-events-none"
      />

      <button
        onClick={handleClose}
        onMouseDown={(e) => e.stopPropagation()}
        className="absolute right-4 top-1/2 -translate-y-1/2 p-1 pt-5 text-black/50 dark:text-white/50 rounded-md cursor-pointer z-10"
        tabIndex={-1}
      >
        <X className="w-5 h-5" />
      </button>
    </div>
  )
}
