import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { SearchBar } from './SearchBar'

describe('SearchBar', () => {
  it('renders with placeholder and reflects value', () => {
    render(<SearchBar value="hello" onChange={() => {}} isDark={false} opacity={0.8} />)
    const input = screen.getByRole('textbox', { name: 'Search...' })
    expect(input).toHaveValue('hello')
  })

  it('calls onChange as the user types', async () => {
    const user = userEvent.setup()
    const onChange = vi.fn()
    render(<SearchBar value="" onChange={onChange} isDark opacity={0.8} />)

    await user.type(screen.getByRole('textbox'), 'abc')
    expect(onChange).toHaveBeenCalledTimes(3)
    expect(onChange).toHaveBeenLastCalledWith('c')
  })

  it('clears the value and invokes onClear', async () => {
    const user = userEvent.setup()
    const onChange = vi.fn()
    const onClear = vi.fn()
    render(
      <SearchBar value="text" onChange={onChange} onClear={onClear} isDark={false} opacity={0.8} />
    )

    await user.click(screen.getByRole('button', { name: 'Clear search' }))
    expect(onChange).toHaveBeenCalledWith('')
    expect(onClear).toHaveBeenCalledOnce()
  })

  it('toggles regex mode', async () => {
    const user = userEvent.setup()
    const onToggleRegex = vi.fn()
    render(
      <SearchBar
        value=""
        onChange={() => {}}
        onToggleRegex={onToggleRegex}
        isRegex={false}
        isDark={false}
        opacity={0.8}
      />
    )

    await user.click(screen.getByRole('button', { name: 'Toggle Regex search' }))
    expect(onToggleRegex).toHaveBeenCalledOnce()
  })

  it('does not show a clear button when value is empty', () => {
    render(<SearchBar value="" onChange={() => {}} isDark={false} opacity={0.8} />)
    expect(screen.queryByRole('button', { name: 'Clear search' })).not.toBeInTheDocument()
  })
})
