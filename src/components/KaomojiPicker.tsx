import { useState, useMemo, useRef, useCallback, useEffect } from 'react'
import { clsx } from 'clsx'
import { invoke } from '@tauri-apps/api/core'
import { KAOMOJI_CATEGORIES, getKaomojis } from '../services/kaomojiService'
import { SearchBar } from './SearchBar'
import type { CustomKaomoji } from '../types/clipboard'

import { PickerLayout } from './common/PickerLayout'
import { CategoryStrip } from './common/CategoryStrip'
import { useResponsiveGrid } from '../hooks/useResponsiveGrid'
import { useKeyboardNavigation } from '../hooks/useKeyboardNavigation'

interface KaomojiPickerProps {
  isDark: boolean
  opacity: number
  customKaomojis?: CustomKaomoji[]
}

export function KaomojiPicker({ isDark, opacity, customKaomojis = [] }: KaomojiPickerProps) {
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null)

  const [categoryFocusedIndex, setCategoryFocusedIndex] = useState(0)
  const [gridFocusedIndex, setGridFocusedIndex] = useState(0)

  const [hoveredKaomoji, setHoveredKaomoji] = useState<{ text: string; category: string } | null>(
    null
  )
  const gridContainerRef = useRef<HTMLDivElement>(null)

  const { containerRef, dimensions } = useResponsiveGrid()

  const columnCount = useMemo(() => {
    const width = dimensions.width
    if (width >= 768) return 4 // md
    if (width >= 640) return 3 // sm
    return 2 // default
  }, [dimensions.width])

  // Reset grid focus
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setGridFocusedIndex(0)
  }, [searchQuery, selectedCategory])

  const kaomojis = useMemo(() => {
    const mappedCustom = customKaomojis.map((c, i) => ({
      id: `custom-${i}`,
      text: c.text,
      category: c.category,
      keywords: c.keywords,
    }))
    return getKaomojis(selectedCategory, searchQuery, mappedCustom)
  }, [selectedCategory, searchQuery, customKaomojis])

  const handlePaste = useCallback(async (text: string) => {
    try {
      await invoke('paste_text', { text })
    } catch (err) {
      console.error('Failed to paste kaomoji', err)
    }
  }, [])

  const handleGridKeyDown = useKeyboardNavigation({
    items: kaomojis,
    columnCount,
    onSelect: (item) => handlePaste(item.text),
    setFocusedIndex: setGridFocusedIndex,
    containerRef: gridContainerRef,
    dataAttributeName: 'data-kaomoji-index',
  })

  return (
    <PickerLayout
      header={
        <SearchBar
          value={searchQuery}
          onChange={(val: string) => {
            setSearchQuery(val)
            setCategoryFocusedIndex(0)
            setGridFocusedIndex(0)
          }}
          placeholder="Search kaomoji..."
          isDark={isDark}
          opacity={opacity}
        />
      }
      subHeader={
        <CategoryStrip
          categories={KAOMOJI_CATEGORIES}
          selectedCategory={selectedCategory}
          onSelectCategory={setSelectedCategory}
          focusedIndex={categoryFocusedIndex}
          setFocusedIndex={setCategoryFocusedIndex}
          isDark={isDark}
          opacity={opacity}
          hasCustom={customKaomojis.length > 0}
        />
      }
      footer={
        hoveredKaomoji ? (
          <>
            <span className="text-xl">{hoveredKaomoji.text}</span>
            <span className="text-xs dark:text-win11-text-secondary text-win11Light-text-secondary truncate">
              {hoveredKaomoji.category}
            </span>
          </>
        ) : (
          <span className="text-xs dark:text-win11-text-tertiary text-win11Light-text-secondary">
            Click to paste kaomoji
          </span>
        )
      }
    >
      {/* Grid Area */}
      <div ref={containerRef} className="h-full overflow-y-scroll p-3 pt-0 scrollbar-win11">
        <div
          ref={gridContainerRef}
          className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2"
          role="grid"
          aria-label="Kaomoji grid"
        >
          {kaomojis.map((item, index) => (
            <button
              key={item.id}
              data-kaomoji-index={index}
              tabIndex={index === gridFocusedIndex ? 0 : -1}
              onClick={() => handlePaste(item.text)}
              onFocus={() => setGridFocusedIndex(index)}
              onKeyDown={(e) => handleGridKeyDown(e, index)}
              onMouseEnter={() => setHoveredKaomoji({ text: item.text, category: item.category })}
              onMouseLeave={() => setHoveredKaomoji(null)}
              className={clsx(
                'h-12 flex items-center justify-center rounded-md text-sm',
                'hover:scale-105 transition-transform duration-100 transform-gpu',
                'border border-transparent hover:border-win11-border-subtle',
                'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent',
                isDark ? 'hover:bg-win11-bg-card-hover' : 'hover:bg-win11Light-bg-card-hover'
              )}
              title={item.category}
              aria-label={`${item.text} - ${item.category}`}
            >
              {item.text}
            </button>
          ))}
          {kaomojis.length === 0 && (
            <div className="col-span-full py-8 text-center text-sm opacity-60">
              No kaomojis found
            </div>
          )}
        </div>
      </div>
    </PickerLayout>
  )
}
