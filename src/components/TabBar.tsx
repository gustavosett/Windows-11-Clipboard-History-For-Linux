import { forwardRef, useRef, useImperativeHandle, useCallback } from 'react'
import { clsx } from 'clsx'
import { ClipboardList, Smile, Image } from 'lucide-react'
import type { ActiveTab } from '../types/clipboard'

interface TabBarProps {
  activeTab: ActiveTab
  onTabChange: (tab: ActiveTab) => void
}

export interface TabBarRef {
  focusFirstTab: () => void
}

const tabs: { id: ActiveTab; label: string; icon: typeof ClipboardList }[] = [
  { id: 'clipboard', label: 'Clipboard', icon: ClipboardList },
  { id: 'emoji', label: 'Emoji', icon: Smile },
  { id: 'gifs', label: 'GIFs', icon: Image },
]

export const TabBar = forwardRef<TabBarRef, TabBarProps>(function TabBar(
  { activeTab, onTabChange },
  ref
) {
  const tabRefs = useRef<(HTMLButtonElement | null)[]>([])

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
        newIndex = (index + 1) % tabs.length
      } else if (e.key === 'ArrowLeft') {
        e.preventDefault()
        newIndex = (index - 1 + tabs.length) % tabs.length
      } else if (e.key === 'Home') {
        e.preventDefault()
        newIndex = 0
      } else if (e.key === 'End') {
        e.preventDefault()
        newIndex = tabs.length - 1
      }

      if (newIndex !== index) {
        tabRefs.current[newIndex]?.focus()
        onTabChange(tabs[newIndex].id)
      }
    },
    [onTabChange]
  )

  return (
    <div
      className="flex items-center gap-1 p-2 px-4 border-b dark:border-win11-border-subtle border-win11Light-border"
      data-tauri-drag-region
      role="tablist"
    >
      {tabs.map((tab, index) => {
        const Icon = tab.icon
        const isActive = activeTab === tab.id

        return (
          <button
            key={tab.id}
            ref={(el) => {
              tabRefs.current[index] = el
            }}
            onClick={() => onTabChange(tab.id)}
            onKeyDown={(e) => handleKeyDown(e, index)}
            role="tab"
            aria-selected={isActive}
            tabIndex={isActive ? 0 : -1}
            className={clsx(
              'no-drag',
              'flex items-center justify-center gap-2 px-4 py-2 rounded-md',
              'text-sm font-medium transition-all duration-150',
              'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent',
              isActive
                ? [
                    'dark:bg-win11-bg-tertiary bg-win11Light-bg-tertiary',
                    'dark:text-win11-text-primary text-win11Light-text-primary',
                  ]
                : [
                    'dark:text-win11-text-secondary text-win11Light-text-secondary',
                    'hover:dark:bg-win11-bg-card-hover hover:bg-win11Light-bg-card-hover',
                  ]
            )}
          >
            <Icon className="w-4 h-4" />
            <span className="hidden sm:inline">{tab.label}</span>
          </button>
        )
      })}
    </div>
  )
})
