import { clsx } from 'clsx'
import { Pin, ChevronDown, History } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import type { ClipboardItem } from '../../types/clipboard'
import { HistoryItem } from '../HistoryItem'

/** Shared visual props for history item rendering. / پراپ‌های دیداری مشترک رندر آیتم‌های تاریخچه. */
export interface HistoryItemVisualProps {
  isDark: boolean
  isCompact: boolean
  secondaryOpacity: number
  enableSmartActions: boolean
  enableUiPolish: boolean
}

interface PinnedSectionProps extends HistoryItemVisualProps {
  /** Pinned items, newest first. / آیتم‌های سنجاق‌شده، از جدیدتر. */
  items: ClipboardItem[]
  expanded: boolean
  onToggleExpanded: () => void
  focusedIndex: number
  onPaste: (id: string) => void
  onDelete: (id: string) => void
  onTogglePin: (id: string) => void
  onFocus: (idx: number) => void
  setItemRef: (index: number, element: HTMLDivElement | null) => void
}

/**
 * Collapsible "Pinned" section rendered inline above the virtualized list.
 * Pinned sets are small by nature, so plain (non-virtualized) rendering
 * keeps focus handling simple; the heavy "Recent" list stays virtualized.
 *
 * بخشِ «سنجاق‌شده»یِ قابل جمع‌شدن که درون‌خطی بالای فهرست مجازی‌شده
 * رسم می‌شود. مجموعهٔ سنجاق‌شده‌ها ذاتاً کوچک است، بنابراین رندر ساده
 * (غیرمجازی) مدیریت فوکوس را ساده نگه می‌دارد؛ فهرست سنگین «اخیر»
 * مجازی می‌ماند.
 */
export function PinnedSection({
  items,
  expanded,
  onToggleExpanded,
  focusedIndex,
  onPaste,
  onDelete,
  onTogglePin,
  onFocus,
  isDark,
  isCompact,
  secondaryOpacity,
  enableSmartActions,
  enableUiPolish,
  setItemRef,
}: PinnedSectionProps) {
  const { t } = useTranslation()

  return (
    <>
      {/* Section header — collapses/expands the pinned block. */}
      {/* سربرگ بخش — بلوک سنجاق‌شده‌ها را جمع/باز می‌کند. */}
      <div className="px-3 pt-2 pb-1 flex-shrink-0">
        <button
          onClick={onToggleExpanded}
          className={clsx(
            'flex items-center gap-1.5 px-1 py-1 text-xs font-medium w-full',
            'dark:text-win11-text-tertiary text-win11Light-text-tertiary',
            'hover:dark:text-win11-text-secondary hover:text-win11Light-text-secondary',
            'rounded transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent'
          )}
          aria-expanded={expanded}
        >
          <Pin size={12} />
          <span>{t('clipboard.pinned_section')}</span>
          <span className="ml-auto opacity-60">{items.length}</span>
          <ChevronDown
            size={12}
            className={clsx('transition-transform duration-150', !expanded && '-rotate-90')}
          />
        </button>
      </div>

      {/* Pinned items (always small, rendered inline). */}
      {/* آیتم‌های سنجاق‌شده (همیشه کم‌تعداد، درون‌خطی رسم می‌شوند). */}
      {expanded && (
        <div className="px-3 pb-1 flex-shrink-0 space-y-2">
          {items.map((item, offset) => (
            <HistoryItem
              key={item.id}
              ref={(el) => setItemRef(offset, el)}
              item={item}
              index={offset}
              isFocused={offset === focusedIndex}
              onPaste={onPaste}
              onDelete={onDelete}
              onTogglePin={onTogglePin}
              onFocus={() => onFocus(offset)}
              isDark={isDark}
              secondaryOpacity={secondaryOpacity}
              isCompact={isCompact}
              enableSmartActions={enableSmartActions}
              enableUiPolish={enableUiPolish}
            />
          ))}
        </div>
      )}
    </>
  )
}

interface RecentSectionLabelProps {
  /** Unpinned item count shown on the right. / تعداد آیتم‌های غیرسنجاق در سمت راست. */
  count: number
}

/**
 * Thin "Recent" divider label above the virtualized list.
 * برچسب جداکنندهٔ نازک «اخیر» بالای فهرست مجازی‌شده.
 */
export function RecentSectionLabel({ count }: RecentSectionLabelProps) {
  const { t } = useTranslation()
  return (
    <div className="px-3 py-1 flex items-center gap-1.5 text-xs dark:text-win11-text-tertiary text-win11Light-text-tertiary flex-shrink-0">
      <History size={12} />
      <span>{t('clipboard.recent_section')}</span>
      <span className="ml-auto opacity-60">{count}</span>
    </div>
  )
}
