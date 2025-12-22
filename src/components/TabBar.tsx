import { forwardRef, useRef, useImperativeHandle, useCallback, useState } from 'react'
import { clsx } from 'clsx'
import { ClipboardList, Smile, Image, Type } from 'lucide-react'
import type { ActiveTab } from '../types/clipboard'

import { getTertiaryBackgroundStyle } from '../utils/themeUtils'

interface TabBarProps {
  activeTab: ActiveTab
  onTabChange: (tab: ActiveTab) => void
  isDark: boolean
  tertiaryOpacity: number

}

export interface TabBarRef {
  focusFirstTab: () => void
}

const ALL_TABS: { id: ActiveTab; label: string; icon: typeof ClipboardList }[] = [
  { id: 'clipboard', label: 'Clipboard', icon: ClipboardList },

  { id: 'emoji', label: 'Emoji', icon: Smile },
  { id: 'kaomoji', label: 'Kaomoji', icon: Type },
  { id: 'gifs', label: 'GIFs', icon: Image },
]

export const TabBar = forwardRef<TabBarRef, TabBarProps>(function TabBar(
  { activeTab, onTabChange, isDark, tertiaryOpacity },
  ref
) {
  const tabRefs = useRef<(HTMLButtonElement | null)[]>([])
  const [hoveredTab, setHoveredTab] = useState<ActiveTab | null>(null)
  
  const visibleTabs = ALL_TABS

  useImperativeHandle(ref, () => ({
    focusFirstTab: () => {
      tabRefs.current[0]?.focus()
    },
  }))

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent, index: number) => {
      let newIndex = index

      if (e.key === 'ArrowRight') {
        e.preventDefault()
        newIndex = (index + 1) % visibleTabs.length
      } else if (e.key === 'ArrowLeft') {
        e.preventDefault()
        newIndex = (index - 1 + visibleTabs.length) % visibleTabs.length
      } else if (e.key === 'Home') {
        e.preventDefault()
        newIndex = 0
      } else if (e.key === 'End') {
        e.preventDefault()
        newIndex = visibleTabs.length - 1
      }

      if (newIndex !== index) {
        tabRefs.current[newIndex]?.focus()
        onTabChange(visibleTabs[newIndex].id)
      }
    },
    [onTabChange, visibleTabs]
  )

  return (
    <div
      className={clsx(
        'flex items-center gap-1 p-2 px-4 border-b',
        isDark ? 'border-win11-border-subtle' : 'border-win11Light-border'
      )}
      data-tauri-drag-region
      role="tablist"
    >
      {visibleTabs.map((tab, index) => {
        const Icon = tab.icon
        const isActive = activeTab === tab.id
        const isHovered = hoveredTab === tab.id

        return (
          <button
            key={tab.id}
            ref={(el) => {
              tabRefs.current[index] = el
            }}
            onClick={() => onTabChange(tab.id)}
            onKeyDown={(e) => handleKeyDown(e, index)}
            onMouseEnter={() => setHoveredTab(tab.id)}
            onMouseLeave={() => setHoveredTab(null)}
            role="tab"
            aria-selected={isActive}
            tabIndex={isActive ? 0 : -1}
            className={clsx(
              'no-drag',
              'flex items-center justify-center gap-2 px-4 py-2 rounded-md',
              'text-sm font-medium transition-all duration-150',
              'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent',
              isActive
                ? [isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary']
                : [isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary']
            )}
            style={
              isActive || isHovered
                ? getTertiaryBackgroundStyle(isDark, tertiaryOpacity)
                : undefined
            }
          >
            <Icon className="w-4 h-4" />
            <span className="hidden sm:inline">{tab.label}</span>
          </button>
        )
      })}
    </div>
  )
})
