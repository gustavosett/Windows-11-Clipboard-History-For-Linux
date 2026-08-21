import { describe, expect, it, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { KeyboardShortcutsSection } from './KeyboardShortcutsSection'

const invokeMock = vi.fn<(...args: unknown[]) => Promise<unknown>>()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}))

describe('KeyboardShortcutsSection', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('lists the three default shortcuts', () => {
    render(<KeyboardShortcutsSection isDark={false} />)
    expect(screen.getByText('Super + V')).toBeInTheDocument()
    expect(screen.getByText('Ctrl + Alt + V')).toBeInTheDocument()
    expect(screen.getByText('Super + .')).toBeInTheDocument()
  })

  it('shows success feedback when registration succeeds', async () => {
    const user = userEvent.setup()
    invokeMock.mockResolvedValue(undefined)
    render(<KeyboardShortcutsSection isDark />)

    await user.click(screen.getByRole('button', { name: /Register Shortkeys in System/ }))
    expect(invokeMock).toHaveBeenCalledWith('register_de_shortcut')
    expect(await screen.findByText('Shortcuts registered successfully!')).toBeInTheDocument()
  })

  it('shows an error message when registration fails', async () => {
    const user = userEvent.setup()
    invokeMock.mockRejectedValue('gsettings missing')
    render(<KeyboardShortcutsSection isDark={false} />)

    await user.click(screen.getByRole('button', { name: /Register Shortkeys in System/ }))
    expect(await screen.findByText('Registration failed')).toBeInTheDocument()
    expect(screen.getByText('gsettings missing')).toBeInTheDocument()
  })
})
