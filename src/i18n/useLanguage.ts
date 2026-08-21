import { useTranslation } from 'react-i18next'
import { useCallback, useEffect } from 'react'
import { changeLanguage, type LangCode } from './config'

export function useLanguage() {
  const { i18n } = useTranslation()
  const currentLang = i18n.language as LangCode

  const setLanguage = useCallback(async (lang: LangCode) => {
    await changeLanguage(lang)
  }, [])

  const isRTL = currentLang === 'fa'

  return { currentLang, setLanguage, isRTL }
}

export function useLanguageEffect(i18n: ReturnType<typeof useTranslation>['i18n']) {
  useEffect(() => {
    const dir = i18n.language === 'fa' ? 'rtl' : 'ltr'
    document.documentElement.dir = dir
    document.documentElement.lang = i18n.language
  }, [i18n.language])
}
