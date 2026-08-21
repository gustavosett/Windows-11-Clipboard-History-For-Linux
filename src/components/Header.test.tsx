import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import '../i18n/config'
import { Header } from './Header'

describe('Header / سربرگ', () => {
  const base = {
    onClearHistory: vi.fn(),
    isDark: false,
    tertiaryOpacity: 0.5,
    isCompact: false,
    onToggleCompact: vi.fn(),
  }

  it('shows a single count when the page is complete', () => {
    render(<Header {...base} itemCount={3} totalCount={3} />)
    expect(screen.getByText('3')).toBeInTheDocument()
  })

  it('shows loaded / total when more pages exist', () => {
    render(<Header {...base} itemCount={100} totalCount={250} />)
    expect(screen.getByText('100 / 250')).toBeInTheDocument()
  })
})
