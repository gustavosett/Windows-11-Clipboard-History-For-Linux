import { clsx } from 'clsx'
import { useTranslation } from 'react-i18next'
import type { CustomKaomoji, UserSettings } from '../../types/clipboard'
import { XIcon } from './icons'
import { SectionCard } from './SectionCard'

interface KaomojiSectionProps {
  settings: UserSettings
  isDark: boolean
  newKaomoji: string
  onNewKaomojiChange: (value: string) => void
  onAdd: () => void
  onRemove: (index: number) => void
}

/** User-defined kaomoji management. */
export function KaomojiSection({
  settings,
  isDark,
  newKaomoji,
  onNewKaomojiChange,
  onAdd,
  onRemove,
}: KaomojiSectionProps) {
  const { t } = useTranslation()

  return (
    <SectionCard
      title={t('settings_page.custom_kaomoji')}
      subtitle={t('settings_page.custom_kaomoji_desc')}
      isDark={isDark}
    >
      <div className="space-y-6">
        {/* Add new */}
        <div className="flex gap-2">
          <input
            type="text"
            value={newKaomoji}
            onChange={(e) => onNewKaomojiChange(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                onAdd()
              }
            }}
            placeholder="( ˘ ³˘)♥"
            className={clsx(
              'flex-1 px-3 py-2 rounded-md border text-sm focus:outline-none focus:ring-2 focus:ring-win11-bg-accent/50 transition-all',
              isDark
                ? 'bg-white/5 border-white/10 text-white placeholder-gray-500'
                : 'bg-gray-50 border-gray-200 text-gray-900 placeholder-gray-400'
            )}
          />
          <button
            onClick={onAdd}
            className="px-4 py-2 bg-win11-bg-accent text-white rounded-md text-sm font-medium hover:opacity-90 active:scale-95 transition-all"
          >
            {t('common.add')}
          </button>
        </div>

        {/* List */}
        {settings.custom_kaomojis.length > 0 ? (
          <div className="grid grid-cols-2 md:grid-cols-3 gap-2 max-h-48 overflow-y-auto custom-scrollbar pr-1">
            {settings.custom_kaomojis.map((item: CustomKaomoji, idx: number) => (
              <div
                key={idx}
                className={clsx(
                  'group flex items-center justify-between px-3 py-2 rounded-md border transition-colors',
                  isDark ? 'bg-white/5 border-white/10' : 'bg-gray-50 border-gray-200'
                )}
              >
                <span className="font-mono text-sm truncate mr-2" title={item.text}>
                  {item.text}
                </span>
                <button
                  onClick={() => onRemove(idx)}
                  className="opacity-0 group-hover:opacity-100 p-1 text-red-500 hover:bg-red-500/10 rounded transition-all focus-visible:opacity-100"
                  title={t('common.delete')}
                  aria-label={t('settings_page.delete_kaomoji', { kaomoji: item.text })}
                >
                  <XIcon />
                </button>
              </div>
            ))}
          </div>
        ) : (
          <div
            className={clsx(
              'text-center py-4 text-sm italic opacity-60',
              isDark ? 'text-gray-500' : 'text-gray-400'
            )}
          >
            {t('settings_page.no_custom_kaomoji')}
          </div>
        )}
      </div>
    </SectionCard>
  )
}
