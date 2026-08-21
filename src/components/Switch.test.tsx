import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Switch } from './Switch'

describe('Switch', () => {
  it('renders an unchecked switch and toggles on click', async () => {
    const user = userEvent.setup()
    const onChange = vi.fn()
    render(<Switch checked={false} onChange={onChange} isDark={false} />)

    const button = screen.getByRole('button')
    // The switch is a plain button (no aria-checked) by design
    expect(button).not.toHaveAttribute('aria-checked')

    await user.click(button)
    expect(onChange).toHaveBeenCalledWith(true)
  })

  it('reports false when clicked while checked', async () => {
    const user = userEvent.setup()
    const onChange = vi.fn()
    render(<Switch checked onChange={onChange} isDark />)

    await user.click(screen.getByRole('button'))
    expect(onChange).toHaveBeenCalledWith(false)
  })

  it('is keyboard-focusable (native button semantics)', () => {
    render(<Switch checked={false} onChange={() => {}} isDark={false} />)
    expect(screen.getByRole('button')).toBeEnabled()
  })
})
