import { useState, useCallback, useMemo, useRef, useLayoutEffect, useEffect } from 'react'
import { ChevronLeft, ChevronRight } from 'lucide-react'
import { clsx } from 'clsx'
import { SearchBar } from './SearchBar'
import { CategoryPill } from './CategoryPill'
import { invoke } from '@tauri-apps/api/core'
import { KAOMOJI_CATEGORIES, getKaomojis } from '../services/kaomojiService'


import type { CustomKaomoji } from '../types/clipboard'

interface KaomojiPickerProps {
  isDark: boolean
  opacity: number
  customKaomojis?: CustomKaomoji[]
}

export function KaomojiPicker({ isDark, opacity, customKaomojis = [] }: KaomojiPickerProps) {
  const [searchQuery, setSearchQuery] = useState('')
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null)
  const [categoryFocusedIndex, setCategoryFocusedIndex] = useState(0)
  const [hoveredKaomoji, setHoveredKaomoji] = useState<{ text: string; category: string } | null>(null)
  const scrollContainerRef = useRef<HTMLDivElement>(null)
  
  // Grid keyboard navigation state
  const [gridFocusedIndex, setGridFocusedIndex] = useState(0)
  const gridContainerRef = useRef<HTMLDivElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const [columnCount, setColumnCount] = useState(2)

  // Calculate column count based on container width
  useLayoutEffect(() => {
    const updateColumnCount = () => {
      if (containerRef.current) {
        const width = containerRef.current.offsetWidth
        // Matches Tailwind breakpoints: grid-cols-2 sm:grid-cols-3 md:grid-cols-4
        if (width >= 768) {
          setColumnCount(4) // md breakpoint
        } else if (width >= 640) {
          setColumnCount(3) // sm breakpoint
        } else {
          setColumnCount(2) // default
        }
      }
    }

    updateColumnCount()
    const observer = new ResizeObserver(updateColumnCount)
    if (containerRef.current) {
      observer.observe(containerRef.current)
    }

    return () => observer.disconnect()
  }, [])

  // Reset grid focus when search or category changes
  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setGridFocusedIndex(0)
  }, [searchQuery, selectedCategory])

  const scrollCategories = (direction: 'left' | 'right') => {
    if (scrollContainerRef.current) {
      const scrollAmount = 200
      scrollContainerRef.current.scrollBy({
        left: direction === 'left' ? -scrollAmount : scrollAmount,
        behavior: 'smooth',
      })
    }
  }

  // Keyboard navigation for categories
  const handleCategoryKeyDown = useCallback(
    (e: React.KeyboardEvent, currentIndex: number) => {
      // Total items = 1 (All) + 1 (Custom if exists) + categories.length
      const hasCustom = customKaomojis.length > 0
      const totalItems = 1 + (hasCustom ? 1 : 0) + KAOMOJI_CATEGORIES.length
      
      let newIndex = currentIndex
      let handled = false

      switch (e.key) {
        case 'ArrowRight':
          if (currentIndex < totalItems - 1) {
            newIndex = currentIndex + 1
            handled = true
          }
          break
        case 'ArrowLeft':
          if (currentIndex > 0) {
            newIndex = currentIndex - 1
            handled = true
          }
          break
        case 'Home':
          newIndex = 0
          handled = true
          break
        case 'End':
          newIndex = totalItems - 1
          handled = true
          break
        case 'Enter':
        case ' ':
          e.preventDefault()
          if (currentIndex === 0) {
            setSelectedCategory(null)
          } else if (hasCustom && currentIndex === 1) {
            setSelectedCategory('Custom')
          } else {
             // If hasCustom, categories start at index 2 (so subtract 2)
             // If no custom, categories start at index 1 (so subtract 1)
             const catIndex = currentIndex - (hasCustom ? 2 : 1)
             setSelectedCategory(KAOMOJI_CATEGORIES[catIndex])
          }
          return
      }

      if (handled) {
        e.preventDefault()
        e.stopPropagation()
        setCategoryFocusedIndex(newIndex)

        // Scroll container if needed
        const container = scrollContainerRef.current
        if (container) {
          const button = container.querySelector(
            `[data-category-index="${newIndex}"]`
          ) as HTMLElement
          if (button) {
            button.focus()
            button.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' })
          }
        }
      }
    },
    [setSelectedCategory, customKaomojis]
  )

  const kaomojis = useMemo(() => {
    // Map CustomKaomoji to Kaomoji if structures differ (they are compatible: text, category, keywords)
    // Add IDs to custom items
    const mappedCustom = customKaomojis.map((c, i) => ({
        id: `custom-${i}`,
        text: c.text,
        category: c.category,
        keywords: c.keywords
    }))
    return getKaomojis(selectedCategory, searchQuery, mappedCustom)
  }, [selectedCategory, searchQuery, customKaomojis])

  const handlePaste = useCallback(
    async (text: string) => {
      try {
        await invoke('paste_text', { text, itemType: 'kaomoji' })
        // No need for toast or close_window here, backend handles it
      } catch (err) {
        console.error('Failed to paste kaomoji', err)
      }
    },
    []
  )

  // Keyboard navigation for kaomoji grid
  const handleGridKeyDown = useCallback(
    (e: React.KeyboardEvent, currentIndex: number) => {
      if (kaomojis.length === 0) return

      let newIndex = currentIndex
      let handled = false

      switch (e.key) {
        case 'ArrowRight':
          if (currentIndex < kaomojis.length - 1) {
            newIndex = currentIndex + 1
            handled = true
          }
          break
        case 'ArrowLeft':
          if (currentIndex > 0) {
            newIndex = currentIndex - 1
            handled = true
          }
          break
        case 'ArrowDown': {
          const nextRowIndex = currentIndex + columnCount
          if (nextRowIndex < kaomojis.length) {
            newIndex = nextRowIndex
            handled = true
          }
          break
        }
        case 'ArrowUp': {
          const prevRowIndex = currentIndex - columnCount
          if (prevRowIndex >= 0) {
            newIndex = prevRowIndex
            handled = true
          }
          break
        }
        case 'Home':
          newIndex = 0
          handled = true
          break
        case 'End':
          newIndex = kaomojis.length - 1
          handled = true
          break
        case 'Enter':
        case ' ':
          e.preventDefault()
          if (kaomojis[currentIndex]) {
            handlePaste(kaomojis[currentIndex].text)
          }
          return
      }

      if (handled) {
        e.preventDefault()
        e.stopPropagation()
        setGridFocusedIndex(newIndex)
        
        // Focus the new element
        const container = gridContainerRef.current
        if (container) {
          const button = container.querySelector(`[data-kaomoji-index="${newIndex}"]`) as HTMLElement
          button?.focus()
        }
      }
    },
    [kaomojis, columnCount, handlePaste]
  )

  return (
    <div className="flex flex-col h-full overflow-hidden select-none">
      {/* Search */}
      <div className="px-3 pt-3 pb-2 flex-shrink-0">
        <SearchBar
           value={searchQuery}
           onChange={setSearchQuery}
           placeholder="Search kaomoji..."
           isDark={isDark}
           opacity={opacity}
        />
      </div>

      {/* Categories */}
      <div className="px-3 pb-2 flex-shrink-0 flex items-center gap-1">
        <button
          onClick={() => scrollCategories('left')}
          className="p-1 rounded-full hover:bg-win11Light-bg-tertiary dark:hover:bg-win11-bg-card-hover text-win11Light-text-secondary dark:text-win11-text-secondary"
          tabIndex={-1}
        >
          <ChevronLeft className="w-4 h-4" />
        </button>

        <div 
          ref={scrollContainerRef}
          className="flex gap-1.5 overflow-x-hidden scroll-smooth flex-1"
          role="tablist"
          aria-label="Kaomoji categories"
        >
          <CategoryPill
              category="All"
              isActive={selectedCategory === null}
              onClick={() => setSelectedCategory(null)}
              tabIndex={categoryFocusedIndex === 0 ? 0 : -1}
              onKeyDown={(e: React.KeyboardEvent) => handleCategoryKeyDown(e, 0)}
              onFocus={() => setCategoryFocusedIndex(0)}
              data-category-index={0}
              isDark={isDark}
              opacity={opacity}
          />
          {customKaomojis.length > 0 && (
              <CategoryPill
                  category="Custom"
                  isActive={selectedCategory === 'Custom'}
                  onClick={() => setSelectedCategory('Custom')}
                  tabIndex={categoryFocusedIndex === 1 ? 0 : -1}
                  onKeyDown={(e: React.KeyboardEvent) => handleCategoryKeyDown(e, 1)}
                  onFocus={() => setCategoryFocusedIndex(1)}
                  data-category-index={1}
                  isDark={isDark}
                  opacity={opacity}
              />
          )}
          {KAOMOJI_CATEGORIES.map((cat, idx) => {
                // Determine the correct index based on whether custom exists
                const hasCustom = customKaomojis.length > 0
                const actualIndex = idx + (hasCustom ? 2 : 1)

                return (
                 <CategoryPill
                    key={cat}
                    category={cat}
                    isActive={selectedCategory === cat}
                    onClick={() => setSelectedCategory(cat)}
                    tabIndex={categoryFocusedIndex === actualIndex ? 0 : -1}
                    onKeyDown={(e: React.KeyboardEvent) => handleCategoryKeyDown(e, actualIndex)}
                    onFocus={() => setCategoryFocusedIndex(actualIndex)}
                    data-category-index={actualIndex}
                    isDark={isDark}
                    opacity={opacity}
                />
               )
          })}
        </div>

        <button
          onClick={() => scrollCategories('right')}
          className="p-1 rounded-full hover:bg-win11Light-bg-tertiary dark:hover:bg-win11-bg-card-hover text-win11Light-text-secondary dark:text-win11-text-secondary"
          tabIndex={-1}
        >
          <ChevronRight className="w-4 h-4" />
        </button>
      </div>

      {/* Grid */}
      <div ref={containerRef} className="flex-1 overflow-y-auto p-3 pt-0 scrollbar-win11">
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
                        "h-12 flex items-center justify-center rounded-md text-sm",
                        "hover:scale-105 transition-transform duration-100",
                        "border border-transparent hover:border-win11-border-subtle",
                        "focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent",
                         isDark ? "hover:bg-win11-bg-card-hover" : "hover:bg-win11Light-bg-card-hover"
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

      {/* Footer with hovered kaomoji info */}
      <div
        className={clsx(
          'px-3 py-2 h-10 flex-shrink-0',
          'border-t dark:border-win11-border-subtle border-win11Light-border',
          'flex items-center gap-2'
        )}
      >
        {hoveredKaomoji ? (
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
        )}
      </div>
    </div>
  )
}
