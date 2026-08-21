import { describe, expect, it, vi, beforeEach } from 'vitest'
import { smartActionService } from './smartActionService'
import { sanitizeOpenUrl } from '../utils/urlSafety'

const invokeMock = vi.fn<(...args: unknown[]) => Promise<unknown>>()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}))

describe('smartActionService', () => {
  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('detects https URLs', () => {
    const actions = smartActionService.detectActions('https://example.com/docs')
    expect(actions.some((a) => a.id === 'open-link')).toBe(true)
  })

  it('upgrades http URLs to https before opening (HTTPS-only policy)', () => {
    // Plain http input is detected, but the produced action data must be the
    // https:// upgrade — the raw http:// URL is never handed to xdg-open.
    // ورودی سادهٔ http شناسایی می‌شود اما دادهٔ action باید نسخهٔ ارتقایافتهٔ
    // https:// باشد — URL خام http هرگز به xdg-open داده نمی‌شود.
    const actions = smartActionService.detectActions('http://example.com/docs')
    const action = actions.find((a) => a.id === 'open-link')
    expect(action).toBeDefined()
    expect(action?.data).toBe('https://example.com/docs')
  })

  it('detects emails', () => {
    const actions = smartActionService.detectActions('user@example.com')
    expect(actions.some((a) => a.id === 'compose-email')).toBe(true)
  })

  it('detects hex and rgb colors', () => {
    expect(smartActionService.detectActions('#0A84FF').some((a) => a.id === 'color-preview')).toBe(
      true
    )
    expect(
      smartActionService.detectActions('rgb(10, 132, 255)').some((a) => a.id === 'color-preview')
    ).toBe(true)
  })

  it('ignores plain text and unsafe URLs', () => {
    expect(smartActionService.detectActions('hello world')).toEqual([])
    expect(smartActionService.detectActions('javascript:alert(1)')).toEqual([])
    expect(sanitizeOpenUrl('javascript:alert(1)')).toBeNull()
  })

  it('ignores overlong text and malformed emails', () => {
    expect(smartActionService.detectActions('a'.repeat(3000))).toEqual([])
    expect(smartActionService.detectActions('not-an-email@')).toEqual([])
  })

  it('executes open-link through the Rust open_safe_url command', async () => {
    const actions = smartActionService.detectActions('https://example.com/docs')
    const action = actions.find((a) => a.id === 'open-link')
    expect(action).toBeDefined()

    invokeMock.mockResolvedValue(undefined)
    await smartActionService.execute(action!)
    expect(invokeMock).toHaveBeenCalledWith('open_safe_url', { url: 'https://example.com/docs' })
  })

  it('executes compose-email through a mailto: URL', async () => {
    const actions = smartActionService.detectActions('user@example.com')
    const action = actions.find((a) => a.id === 'compose-email')
    expect(action).toBeDefined()

    invokeMock.mockResolvedValue(undefined)
    await smartActionService.execute(action!)
    expect(invokeMock).toHaveBeenCalledWith('open_safe_url', { url: 'mailto:user@example.com' })
  })

  it('throws when asked to open a blocked URL', async () => {
    await expect(
      smartActionService.execute({ id: 'open-link', label: 'Open Link', data: 'http://127.0.0.1/' })
    ).rejects.toThrow('Blocked unsafe URL')
    expect(invokeMock).not.toHaveBeenCalled()
  })
})
