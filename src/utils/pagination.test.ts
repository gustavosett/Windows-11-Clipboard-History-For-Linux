import { describe, it, expect } from 'vitest'
import {
  MAX_PAGE_SIZE,
  DEFAULT_PAGE_SIZE,
  clampPageSize,
  clampOffset,
  hasNextPage,
  mergePageById,
} from './pagination'

describe('clampPageSize / محدودسازی اندازهٔ صفحه', () => {
  it('defaults when the request is missing or invalid', () => {
    expect(clampPageSize(undefined)).toBe(DEFAULT_PAGE_SIZE)
    expect(clampPageSize(null)).toBe(DEFAULT_PAGE_SIZE)
    expect(clampPageSize(Number.NaN)).toBe(DEFAULT_PAGE_SIZE)
  })

  it('clamps into 1..=MAX_PAGE_SIZE', () => {
    expect(clampPageSize(0)).toBe(1)
    expect(clampPageSize(-5)).toBe(1)
    expect(clampPageSize(20)).toBe(20)
    expect(clampPageSize(MAX_PAGE_SIZE)).toBe(MAX_PAGE_SIZE)
    expect(clampPageSize(MAX_PAGE_SIZE + 1)).toBe(MAX_PAGE_SIZE)
  })

  it('truncates fractional sizes', () => {
    expect(clampPageSize(10.9)).toBe(10)
  })
})

describe('clampOffset / محدودسازی آفست', () => {
  it('defaults to zero and never goes negative', () => {
    expect(clampOffset(undefined)).toBe(0)
    expect(clampOffset(-3)).toBe(0)
    expect(clampOffset(42)).toBe(42)
    expect(clampOffset(7.6)).toBe(7)
  })
})

describe('hasNextPage / وجود صفحهٔ بعد', () => {
  it('knows when more items exist', () => {
    expect(hasNextPage(10, 0, 5)).toBe(true)
    expect(hasNextPage(10, 5, 5)).toBe(false)
    expect(hasNextPage(10, 9, 5)).toBe(false)
    expect(hasNextPage(0, 0, 5)).toBe(false)
  })
})

describe('mergePageById / ادغام صفحه‌ها', () => {
  it('appends only unseen ids and keeps order', () => {
    const existing = [{ id: 'a' }, { id: 'b' }]
    const merged = mergePageById(existing, [
      { id: 'b' },
      { id: 'c' },
      { id: 'a' },
      { id: 'd' },
    ])
    expect(merged.map((i) => i.id)).toEqual(['a', 'b', 'c', 'd'])
  })

  it('returns the original list for an empty page', () => {
    const existing = [{ id: 'a' }]
    expect(mergePageById(existing, [])).toBe(existing)
  })
})
