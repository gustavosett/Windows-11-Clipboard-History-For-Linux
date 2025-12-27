import { useRef, useCallback } from 'react'
import { ChevronLeft, ChevronRight } from 'lucide-react'
import { CategoryPill } from '../CategoryPill'

interface CategoryStripProps {
  categories: string[] | readonly string[]
  selectedCategory: string | null
  onSelectCategory: (cat: string | null) => void
  focusedIndex: number
  setFocusedIndex: (index: number) => void
  isDark: boolean
  opacity: number
  hasCustom?: boolean
}

export function CategoryStrip({
  categories,
  selectedCategory,
  onSelectCategory,
  focusedIndex,
  setFocusedIndex,
  isDark,
  opacity,
  hasCustom = false,
}: CategoryStripProps) {
  const scrollContainerRef = useRef<HTMLDivElement>(null)

  const scroll = (direction: 'left' | 'right') => {
    if (scrollContainerRef.current) {
      const scrollAmount = 200
      scrollContainerRef.current.scrollBy({
        left: direction === 'left' ? -scrollAmount : scrollAmount,
        behavior: 'smooth',
      })
    }
  }

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent, currentIndex: number) => {
      const totalItems = 1 + (hasCustom ? 1 : 0) + categories.length
      let newIndex = currentIndex
      let handled = false

      switch (e.key) {
        case 'ArrowRight':
          if (currentIndex < totalItems - 1) {
            newIndex++
            handled = true
          }
          break
        case 'ArrowLeft':
          if (currentIndex > 0) {
            newIndex--
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
          if (currentIndex === 0) onSelectCategory(null)
          else if (hasCustom && currentIndex === 1) onSelectCategory('Custom')
          else {
            const catIndex = currentIndex - (hasCustom ? 2 : 1)
            onSelectCategory(categories[catIndex])
          }
          return
      }

      if (handled) {
        e.preventDefault()
        e.stopPropagation()
        setFocusedIndex(newIndex)

        const container = scrollContainerRef.current
        if (container) {
          const button = container.querySelector(
            `[data-category-index="${newIndex}"]`
          ) as HTMLElement
          button?.focus()
          button?.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' })
        }
      }
    },
    [categories, hasCustom, onSelectCategory, setFocusedIndex]
  )

  return (
    <div className="flex items-center gap-1">
      <button
        onClick={() => scroll('left')}
        className="p-1 rounded-full hover:bg-win11Light-bg-tertiary dark:hover:bg-win11-bg-card-hover text-win11Light-text-secondary dark:text-win11-text-secondary"
        tabIndex={-1}
      >
        <ChevronLeft className="w-4 h-4" />
      </button>

      <div
        ref={scrollContainerRef}
        className="flex gap-1.5 overflow-x-hidden scroll-smooth flex-1"
        role="tablist"
      >
        {/* All */}
        <CategoryPill
          category="All"
          isActive={selectedCategory === null}
          onClick={() => onSelectCategory(null)}
          tabIndex={focusedIndex === 0 ? 0 : -1}
          onKeyDown={(e) => handleKeyDown(e, 0)}
          onFocus={() => setFocusedIndex(0)}
          data-category-index={0}
          isDark={isDark}
          opacity={opacity}
        />

        {/* Custom (Kaomoji only) */}
        {hasCustom && (
          <CategoryPill
            category="Custom"
            isActive={selectedCategory === 'Custom'}
            onClick={() => onSelectCategory('Custom')}
            tabIndex={focusedIndex === 1 ? 0 : -1}
            onKeyDown={(e) => handleKeyDown(e, 1)}
            onFocus={() => setFocusedIndex(1)}
            data-category-index={1}
            isDark={isDark}
            opacity={opacity}
          />
        )}

        {/* Dynamic Categories */}
        {categories.map((cat, idx) => {
          const actualIndex = idx + (hasCustom ? 2 : 1)
          return (
            <CategoryPill
              key={cat}
              category={cat}
              isActive={selectedCategory === cat}
              onClick={() => onSelectCategory(cat)}
              tabIndex={focusedIndex === actualIndex ? 0 : -1}
              onKeyDown={(e) => handleKeyDown(e, actualIndex)}
              onFocus={() => setFocusedIndex(actualIndex)}
              data-category-index={actualIndex}
              isDark={isDark}
              opacity={opacity}
            />
          )
        })}
      </div>

      <button
        onClick={() => scroll('right')}
        className="p-1 rounded-full hover:bg-win11Light-bg-tertiary dark:hover:bg-win11-bg-card-hover text-win11Light-text-secondary dark:text-win11-text-secondary"
        tabIndex={-1}
      >
        <ChevronRight className="w-4 h-4" />
      </button>
    </div>
  )
}
