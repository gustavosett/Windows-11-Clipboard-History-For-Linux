import { describe, expect, it } from 'vitest'
import en from '../locales/en.json'
import fa from '../locales/fa.json'
import i18n, { applyWindowLanguagePolicy, changeLanguage } from './config'

function flatten(value: unknown, prefix = ''): Record<string, string> {
  if (value === null || value === undefined) return { [prefix]: '' }
  if (typeof value === 'string') return { [prefix]: value }
  if (typeof value === 'number' || typeof value === 'boolean') {
    return { [prefix]: String(value) }
  }
  if (Array.isArray(value)) return { [prefix]: JSON.stringify(value) }

  return Object.entries(value).reduce<Record<string, string>>((result, [key, child]) => {
    const path = prefix ? `${prefix}.${key}` : key
    return { ...result, ...flatten(child, path) }
  }, {})
}

describe('locale catalogs', () => {
  it('keeps English and Persian translation keys in parity', () => {
    const englishKeys = Object.keys(flatten(en)).sort()
    const persianKeys = Object.keys(flatten(fa)).sort()

    expect(persianKeys).toEqual(englishKeys)
  })

  it('does not contain empty translations', () => {
    for (const [key, value] of [...Object.entries(flatten(en)), ...Object.entries(flatten(fa))]) {
      expect(value.trim(), `${key} must not be empty`).not.toBe('')
    }
  })

  it('keeps the main popup English/LTR without overwriting the saved preference', async () => {
    await changeLanguage('fa')
    await applyWindowLanguagePolicy('main')

    expect(i18n.language).toBe('en')
    expect(document.documentElement.lang).toBe('en')
    expect(document.documentElement.dir).toBe('ltr')
    expect(localStorage.getItem('i18nextLng')).toBe('fa')

    await changeLanguage('en')
  })

  it('allows Persian RTL on Setup and Settings surfaces', async () => {
    await changeLanguage('fa')
    await applyWindowLanguagePolicy('setup')

    expect(i18n.language).toBe('fa')
    expect(document.documentElement.dir).toBe('rtl')

    await changeLanguage('en')
  })
})
