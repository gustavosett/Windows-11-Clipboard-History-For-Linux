import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import '../i18n/config'
import { EmptyState } from './EmptyState'

describe('EmptyState', () => {
  it('renders the empty copy and Super+V hint', () => {
    render(<EmptyState isDark={false} />)
    expect(screen.getByRole('status')).toBeInTheDocument()
    expect(screen.getByText(/clipboard history is empty/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/Super \+ V/i)).toBeInTheDocument()
    expect(screen.getByText('Super')).toBeInTheDocument()
  })

  it('renders in dark mode without crashing', () => {
    render(<EmptyState isDark />)
    expect(screen.getByRole('status')).toBeInTheDocument()
  })
})
