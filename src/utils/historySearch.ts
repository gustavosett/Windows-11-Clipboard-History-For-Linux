import type { ClipboardItem } from '../types/clipboard'

const MAX_REGEX_LENGTH = 80
const NESTED_QUANTIFIER = /[+\-*?]{2,}|\(\?|\(\*|\+\{|\*\{|\([^)]*[+*?][^)]*\)[+*?]/

export function isSafeRegexPattern(pattern: string): boolean {
  if (!pattern || pattern.length > MAX_REGEX_LENGTH) return false
  if (NESTED_QUANTIFIER.test(pattern)) return false
  try {
    const compiled = new RegExp(pattern, 'i')
    return compiled.source.length > 0
  } catch {
    return false
  }
}

export function itemSearchText(item: ClipboardItem): string | null {
  if (item.content.type === 'Text') return item.content.data
  if (item.content.type === 'RichText') return item.content.data.plain
  return null
}

export function filterHistory(
  history: ClipboardItem[],
  searchQuery: string,
  isRegexMode: boolean
): ClipboardItem[] {
  if (!searchQuery) return history

  if (isRegexMode) {
    if (!isSafeRegexPattern(searchQuery)) return []
    let regex: RegExp
    try {
      regex = new RegExp(searchQuery, 'i')
    } catch {
      return []
    }
    return history.filter((item) => {
      const text = itemSearchText(item)
      return text ? regex.test(text) : false
    })
  }

  const needle = searchQuery.toLowerCase()
  return history.filter((item) => {
    const text = itemSearchText(item)
    return text ? text.toLowerCase().includes(needle) : false
  })
}
