import { describe, it, expect } from 'vitest'
import { isSafeRegexPattern, itemSearchText, filterHistory } from './historySearch'
import type { ClipboardItem } from '../types/clipboard'

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function makeTextItem(text: string, pinned = false): ClipboardItem {
  return {
    id: crypto.randomUUID(),
    content: { type: 'Text', data: text },
    timestamp: new Date().toISOString(),
    pinned,
    preview: text.slice(0, 100),
  }
}

function makeRichTextItem(plain: string): ClipboardItem {
  return {
    id: crypto.randomUUID(),
    content: {
      type: 'RichText',
      data: { plain, html: `<p>${plain}</p>` },
    },
    timestamp: new Date().toISOString(),
    pinned: false,
    preview: plain.slice(0, 100),
  }
}

function makeImageItem(): ClipboardItem {
  return {
    id: crypto.randomUUID(),
    content: {
      type: 'Image',
      data: { base64: 'abc123', width: 100, height: 100 },
    },
    timestamp: new Date().toISOString(),
    pinned: false,
    preview: 'Image (100x100) #12345',
  }
}

// ---------------------------------------------------------------------------
// isSafeRegexPattern
// ---------------------------------------------------------------------------

describe('isSafeRegexPattern', () => {
  it('accepts simple patterns', () => {
    expect(isSafeRegexPattern('hello')).toBe(true)
    expect(isSafeRegexPattern('\\d+')).toBe(true)
    expect(isSafeRegexPattern('[a-z]+')).toBe(true)
  })

  it('rejects empty patterns', () => {
    expect(isSafeRegexPattern('')).toBe(false)
  })

  it('rejects patterns exceeding 80 characters', () => {
    expect(isSafeRegexPattern('a'.repeat(81))).toBe(false)
  })

  it('rejects nested quantifiers (ReDoS risk)', () => {
    expect(isSafeRegexPattern('a++')).toBe(false)
    expect(isSafeRegexPattern('a**')).toBe(false)
    expect(isSafeRegexPattern('a??')).toBe(false)
    expect(isSafeRegexPattern('(a+)+')).toBe(false)
  })

  it('rejects invalid regex syntax', () => {
    expect(isSafeRegexPattern('[invalid')).toBe(false)
    expect(isSafeRegexPattern('(unclosed')).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// itemSearchText
// ---------------------------------------------------------------------------

describe('itemSearchText', () => {
  it('returns text for Text items', () => {
    const item = makeTextItem('hello world')
    expect(itemSearchText(item)).toBe('hello world')
  })

  it('returns plain text for RichText items', () => {
    const item = makeRichTextItem('hello rich')
    expect(itemSearchText(item)).toBe('hello rich')
  })

  it('returns null for Image items', () => {
    const item = makeImageItem()
    expect(itemSearchText(item)).toBeNull()
  })
})

// ---------------------------------------------------------------------------
// filterHistory
// ---------------------------------------------------------------------------

describe('filterHistory', () => {
  const history: ClipboardItem[] = [
    makeTextItem('Hello World'),
    makeTextItem('foo bar baz'),
    makeRichTextItem('Hello Rich Text'),
    makeImageItem(),
    makeTextItem('regex test 123'),
  ]

  it('returns all items when query is empty', () => {
    expect(filterHistory(history, '', false)).toHaveLength(5)
  })

  it('filters by plain text (case-insensitive)', () => {
    const result = filterHistory(history, 'hello', false)
    expect(result).toHaveLength(2) // "Hello World" + "Hello Rich Text"
  })

  it('filters by regex mode', () => {
    const result = filterHistory(history, '\\d+', true)
    expect(result).toHaveLength(1) // "regex test 123"
  })

  it('returns empty for unsafe regex', () => {
    const result = filterHistory(history, 'a++', true)
    expect(result).toHaveLength(0)
  })

  it('skips image items (no searchable text)', () => {
    const result = filterHistory(history, 'image', false)
    expect(result).toHaveLength(0)
  })

  it('handles case-insensitive search', () => {
    const result = filterHistory(history, 'HELLO', false)
    expect(result).toHaveLength(2)
  })

  it('returns empty when nothing matches', () => {
    const result = filterHistory(history, 'nonexistent', false)
    expect(result).toHaveLength(0)
  })
})
