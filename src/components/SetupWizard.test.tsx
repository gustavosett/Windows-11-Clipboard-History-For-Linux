import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { WizardButton } from './setup/WizardButton'
import { StatusCard } from './setup/StatusCard'

const buttonContext = {
  hoveredButton: null,
  setHoveredButton: vi.fn(),
  isDark: false,
  tertiaryOpacity: 0.85,
}

describe('WizardButton', () => {
  it('renders its label and fires onClick', async () => {
    const user = userEvent.setup()
    const onClick = vi.fn()
    render(
      <WizardButton {...buttonContext} id="start" onClick={onClick}>
        Get started
      </WizardButton>
    )

    const button = screen.getByRole('button', { name: 'Get started' })
    await user.click(button)
    expect(onClick).toHaveBeenCalledTimes(1)
  })

  it('is disabled and does not fire when disabled', async () => {
    const user = userEvent.setup()
    const onClick = vi.fn()
    render(
      <WizardButton {...buttonContext} id="fix" onClick={onClick} disabled>
        Fix
      </WizardButton>
    )

    const button = screen.getByRole('button', { name: 'Fix' })
    expect(button).toBeDisabled()
    await user.click(button)
    expect(onClick).not.toHaveBeenCalled()
  })
})

describe('StatusCard', () => {
  it.each(['success', 'warning', 'error'] as const)('renders %s feedback', (type) => {
    render(
      <StatusCard type={type} isDark={false}>
        message-text
      </StatusCard>
    )
    expect(screen.getByText('message-text')).toBeInTheDocument()
  })
})
