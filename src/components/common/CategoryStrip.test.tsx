import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { CategoryStrip } from './CategoryStrip'

describe('CategoryStrip', () => {
  const categories = ['Emoji', 'Symbols', 'Kaomoji']

  it('renders All plus every category', () => {
    render(
      <CategoryStrip
        categories={categories}
        selectedCategory={null}
        onSelectCategory={() => {}}
        focusedIndex={0}
        setFocusedIndex={() => {}}
        isDark={false}
        opacity={0.8}
      />
    )
    expect(screen.getByText('All')).toBeInTheDocument()
    for (const cat of categories) {
      expect(screen.getByText(cat)).toBeInTheDocument()
    }
  })

  it('selects a category on click', async () => {
    const user = userEvent.setup()
    const onSelect = vi.fn()
    render(
      <CategoryStrip
        categories={categories}
        selectedCategory={null}
        onSelectCategory={onSelect}
        focusedIndex={0}
        setFocusedIndex={() => {}}
        isDark={false}
        opacity={0.8}
      />
    )

    await user.click(screen.getByText('Symbols'))
    expect(onSelect).toHaveBeenCalledWith('Symbols')
  })

  it('supports arrow-key navigation and Enter to select', async () => {
    const user = userEvent.setup()
    const onSelect = vi.fn()
    const setFocused = vi.fn()
    render(
      <CategoryStrip
        categories={categories}
        selectedCategory={null}
        onSelectCategory={onSelect}
        focusedIndex={0}
        setFocusedIndex={setFocused}
        isDark
        opacity={0.8}
      />
    )

    const allPill = screen.getByText('All')
    allPill.focus()

    await user.keyboard('{ArrowRight}')
    expect(setFocused).toHaveBeenCalledWith(1)

    // Home returns focus to "All", Enter selects null (show everything)
    await user.keyboard('{Home}{Enter}')
    expect(onSelect).toHaveBeenCalledWith(null)
  })

  it('renders a Custom pill when hasCustom is set', () => {
    render(
      <CategoryStrip
        categories={categories}
        selectedCategory="Custom"
        onSelectCategory={() => {}}
        focusedIndex={1}
        setFocusedIndex={() => {}}
        isDark={false}
        opacity={0.8}
        hasCustom
      />
    )
    expect(screen.getByText('Custom')).toBeInTheDocument()
  })
})
