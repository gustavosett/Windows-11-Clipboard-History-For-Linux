/**
 * Symbol Service
 * Provides static data for symbols and special characters
 */
import symbolsData from '../data/symbols.json'

export interface SymbolItem {
  char: string
  name: string
  category: string
  keywords: string[]
}

export const SYMBOL_CATEGORIES = [
  'General Punctuation',
  'Technical Symbols',
  'Currency Symbols',
  'Latin Symbols',
  'Letterlike Symbols',
  'Greek Symbols',
  'Math Symbols',
  'Geometric Symbols',
  'Dingbats',
  'Arrows',
  'Box Drawing',
  'Block Elements',
  'Miscellaneous Symbols',
  'Musical Symbols',
] as const

export type SymbolCategory = (typeof SYMBOL_CATEGORIES)[number]

// prettier-ignore
export const SYMBOLS: SymbolItem[] = symbolsData as SymbolItem[];

export function getSymbols(category?: string | null, searchQuery?: string): SymbolItem[] {
  let filtered = SYMBOLS

  if (category && category !== 'All') {
    filtered = filtered.filter((s) => s.category === category)
  }

  if (searchQuery) {
    const query = searchQuery.toLowerCase()
    filtered = filtered.filter(
      (s) =>
        s.name.toLowerCase().includes(query) ||
        s.char.includes(query) ||
        s.keywords.some((k) => k.toLowerCase().includes(query))
    )
  }

  return filtered
}

export function getSymbolCategories(): string[] {
  return [...SYMBOL_CATEGORIES]
}
