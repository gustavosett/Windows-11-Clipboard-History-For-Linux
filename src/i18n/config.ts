import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import fa from '../locales/fa.json'
import en from '../locales/en.json'

const SUPPORTED_LANGS = ['fa', 'en'] as const
export type LangCode = (typeof SUPPORTED_LANGS)[number]
export const DEFAULT_LANG: LangCode = 'en'

const detectLanguage = (): LangCode => {
  try {
    const stored = localStorage.getItem('i18nextLng')
    if (stored && SUPPORTED_LANGS.includes(stored as LangCode)) {
      return stored as LangCode
    }
  } catch {
    /* localStorage may be blocked */
  }
  return DEFAULT_LANG
}

// Module-level init is intentionally fire-and-forget; React-i18next suspends
// rendering until the resources are ready and errors are logged by i18next.
// راه‌اندازی سطح ماژول آگاهانه fire-and-forget است؛ React-i18next رندر را تا
// آماده‌شدن منابع معلق می‌کند و خطاها را خود i18next لاگ می‌کند.
void i18n.use(initReactI18next).init({
  resources: { fa: { translation: fa }, en: { translation: en } },
  lng: detectLanguage(),
  fallbackLng: 'en',
  interpolation: { escapeValue: false },
  returnObjects: false,
  returnNull: false,
})

export const applyDocumentLang = (lang: LangCode) => {
  document.documentElement.lang = lang
  document.documentElement.dir = lang === 'fa' ? 'rtl' : 'ltr'
}

applyDocumentLang(detectLanguage())

/// Apply the per-window language policy without persisting a preference.
/// اعمال سیاست زبان هر پنجره بدون ذخیرهٔ ترجیح جدید.
export const applyWindowLanguagePolicy = async (windowLabel: string) => {
  if (windowLabel === 'main') {
    await i18n.changeLanguage('en')
    document.documentElement.lang = 'en'
    document.documentElement.dir = 'ltr'
    return
  }
  applyDocumentLang(i18n.language === 'fa' ? 'fa' : 'en')
}

export const changeLanguage = async (lang: LangCode) => {
  await i18n.changeLanguage(lang)
  applyDocumentLang(lang)
  try {
    localStorage.setItem('i18nextLng', lang)
  } catch {
    /* ignore */
  }
}

export default i18n
