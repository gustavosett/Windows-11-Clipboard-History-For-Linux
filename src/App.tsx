import { useState, useCallback, useEffect } from 'react'
import { clsx } from 'clsx'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useClipboardHistory } from './hooks/useClipboardHistory'
import { useDarkMode } from './hooks/useDarkMode'
import { HistoryItem } from './components/HistoryItem'
import { TabBar } from './components/TabBar'
import { Header } from './components/Header'
import { EmptyState } from './components/EmptyState'
import { DragHandle } from './components/DragHandle'
import { EmojiPicker } from './components/EmojiPicker'
import { GifPicker } from './components/GifPicker'
import type { ActiveTab } from './types/clipboard'
import { invoke } from '@tauri-apps/api/core'

/**
 * Main App Component - Windows 11 Clipboard History Manager
 */
function App() {
  const [activeTab, setActiveTab] = useState<ActiveTab>('clipboard')
  const isDark = useDarkMode()

  const { history, isLoading, clearHistory, deleteItem, togglePin, pasteItem } =
    useClipboardHistory()

  // Handle ESC key to close/hide window
  useEffect(() => {
    const handleKeyDown = async (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault()
        try {
          await getCurrentWindow().hide()
        } catch (err) {
          console.error('Failed to hide window:', err)
        }
      }
    }

    globalThis.addEventListener('keydown', handleKeyDown)
    return () => globalThis.removeEventListener('keydown', handleKeyDown)
  }, [])

  // Handle tab change
  const handleTabChange = useCallback((tab: ActiveTab) => {
    setActiveTab(tab)
  }, [])

  const handleMouseEnter = () => {
    invoke('set_mouse_state', { inside: true }).catch(console.error)
  }

  const handleMouseLeave = () => {
    invoke('set_mouse_state', { inside: false }).catch(console.error)
  }

  // Render content based on active tab
  const renderContent = () => {
    switch (activeTab) {
      case 'clipboard':
        if (isLoading) {
          return (
            <div className="flex items-center justify-center h-full">
              <div className="w-6 h-6 border-2 border-win11-bg-accent border-t-transparent rounded-full animate-spin" />
            </div>
          )
        }

        if (history.length === 0) {
          return <EmptyState />
        }

        return (
          <>
            <Header
              onClearHistory={clearHistory}
              itemCount={history.filter((i) => !i.pinned).length}
            />
            <div className="flex flex-col gap-2 p-3">
              {history.map((item, index) => (
                <HistoryItem
                  key={item.id}
                  item={item}
                  index={index}
                  onPaste={pasteItem}
                  onDelete={deleteItem}
                  onTogglePin={togglePin}
                />
              ))}
            </div>
          </>
        )

      case 'emoji':
        return <EmojiPicker />

      case 'gifs':
        return <GifPicker />

      default:
        return null
    }
  }

  return (
    <div
      className={clsx(
        'h-screen w-screen overflow-hidden flex flex-col rounded-win11-lg',
        isDark ? 'glass-effect' : 'glass-effect-light',
        isDark ? 'bg-win11-acrylic-bg' : 'bg-win11Light-acrylic-bg',
        isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
      )}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {/* Drag Handle */}
      <DragHandle />

      {/* Header with title and actions */}

      {/* Tab bar */}
      <TabBar activeTab={activeTab} onTabChange={handleTabChange} />

      {/* Scrollable content area */}
      <div
        className={clsx(
          'flex-1',
          // Only use scrollbar for non-emoji tabs, emoji has its own virtualized scrolling
          activeTab === 'emoji' ? 'overflow-hidden' : 'overflow-y-auto scrollbar-win11'
        )}
      >
        {renderContent()}
      </div>
    </div>
  )
}

export default App
