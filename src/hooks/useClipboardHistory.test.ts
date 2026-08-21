import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'

// Mock Tauri APIs before importing the hook
const listeners: Record<string, (payload: unknown) => void> = {}
const invokeMock = vi.fn<(...args: unknown[]) => Promise<unknown>>()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn((event: string, cb: unknown) => {
    listeners[event] = cb as (payload: unknown) => void
    return Promise.resolve(() => {})
  }),
}))

import { useClipboardHistory } from './useClipboardHistory'

interface MockItem {
  id: string
  content: { type: 'Text'; data: string }
  timestamp: string
  pinned: boolean
  preview: string
}

const makeItem = (id: string, pinned = false): MockItem => ({
  id,
  content: { type: 'Text', data: id },
  timestamp: '2026-08-20T00:00:00Z',
  pinned,
  preview: id,
})

describe('useClipboardHistory', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    Object.keys(listeners).forEach((k) => delete listeners[k])
    invokeMock.mockResolvedValue({ items: [], total: 0, limit: 100, offset: 0 })
  })

  it('loads history on mount via get_history_page', async () => {
    invokeMock.mockResolvedValueOnce({
      items: [makeItem('1')],
      total: 1,
      limit: 100,
      offset: 0,
    })
    const { result } = renderHook(() => useClipboardHistory())

    await waitFor(() => expect(result.current.history).toHaveLength(1))
    expect(invokeMock).toHaveBeenCalledWith('get_history_page', {
      limit: 100,
      offset: 0,
    })
    expect(result.current.history[0].id).toBe('1')
  })

  it('clearHistory removes unpinned items but keeps pinned ones', async () => {
    invokeMock.mockResolvedValueOnce({
      items: [makeItem('pinned', true), makeItem('normal')],
      total: 2,
      limit: 100,
      offset: 0,
    })
    const { result } = renderHook(() => useClipboardHistory())
    await waitFor(() => expect(result.current.history).toHaveLength(2))

    invokeMock.mockResolvedValueOnce(undefined)
    await act(async () => {
      await result.current.clearHistory()
    })

    expect(result.current.history.map((i) => i.id)).toEqual(['pinned'])
  })

  it('deleteItem removes the requested item', async () => {
    invokeMock.mockResolvedValueOnce({
      items: [makeItem('1'), makeItem('2')],
      total: 2,
      limit: 100,
      offset: 0,
    })
    const { result } = renderHook(() => useClipboardHistory())
    await waitFor(() => expect(result.current.history).toHaveLength(2))

    invokeMock.mockResolvedValueOnce(undefined)
    await act(async () => {
      await result.current.deleteItem('1')
    })

    expect(result.current.history.map((i) => i.id)).toEqual(['2'])
  })

  it('togglePin groups the item with pinned items (newest pin last in group)', async () => {
    invokeMock.mockResolvedValueOnce({
      items: [makeItem('a'), makeItem('b', true)],
      total: 2,
      limit: 100,
      offset: 0,
    })
    const { result } = renderHook(() => useClipboardHistory())
    await waitFor(() => expect(result.current.history).toHaveLength(2))

    invokeMock.mockResolvedValueOnce({ ...makeItem('a'), pinned: true })
    await act(async () => {
      await result.current.togglePin('a')
    })

    // Pinned group first; the newly pinned item joins at the end of the group
    expect(result.current.history.map((i) => i.id)).toEqual(['b', 'a'])
  })

  it('loadMore fetches the next page and merges by id', async () => {
    invokeMock.mockResolvedValueOnce({
      items: [makeItem('1')],
      total: 2,
      limit: 1,
      offset: 0,
    })
    const { result } = renderHook(() => useClipboardHistory({ pageSize: 1 }))
    await waitFor(() => expect(result.current.history).toHaveLength(1))

    invokeMock.mockResolvedValueOnce({
      items: [makeItem('2')],
      total: 2,
      limit: 1,
      offset: 1,
    })
    await act(async () => {
      await result.current.loadMore()
    })
    expect(result.current.history.map((i) => i.id)).toEqual(['1', '2'])
  })

  it('applies clipboard-changed events pushed from the backend', async () => {
    const { result } = renderHook(() => useClipboardHistory())
    await waitFor(() => expect(result.current.history).toHaveLength(0))

    act(() => {
      listeners['clipboard-changed']?.({ payload: makeItem('fresh') })
    })

    expect(result.current.history.map((i) => i.id)).toEqual(['fresh'])
  })

  it('unpinning restores timestamp ordering', async () => {
    const older = { ...makeItem('old'), timestamp: '2026-08-19T00:00:00Z' }
    const newer = { ...makeItem('new'), timestamp: '2026-08-20T00:00:00Z' }
    invokeMock.mockResolvedValueOnce({
      items: [older, newer],
      total: 2,
      limit: 100,
      offset: 0,
    })
    const { result } = renderHook(() => useClipboardHistory())
    await waitFor(() => expect(result.current.history).toHaveLength(2))

    invokeMock.mockResolvedValueOnce({ ...older, pinned: true })
    await act(async () => {
      await result.current.togglePin('old')
    })
    // pinned: old first
    expect(result.current.history.map((i) => i.id)).toEqual(['old', 'new'])

    invokeMock.mockResolvedValueOnce({ ...older, pinned: false })
    await act(async () => {
      await result.current.togglePin('old')
    })
    // unpinned: new first (newest timestamp)
    expect(result.current.history.map((i) => i.id)).toEqual(['new', 'old'])
  })
})
