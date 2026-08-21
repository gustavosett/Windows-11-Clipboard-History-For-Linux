import type { ClipboardItem } from '../../types/clipboard'
import { HistoryItem } from '../HistoryItem'

/**
 * Props shared by every rendered history row (virtualized or inline).
 * The virtualizer forwards these as `rowProps` on each row render.
 *
 * پراپ‌های مشترک همهٔ ردیف‌های تاریخچه (مجازی‌شده یا درون‌خطی).
 * مجازی‌ساز این‌ها را به‌صورت `rowProps` به هر ردیف می‌فرستد.
 */
export interface RowData {
  items: ClipboardItem[]
  onPaste: (id: string) => void
  onDelete: (id: string) => void
  onTogglePin: (id: string) => void
  onFocus: (idx: number) => void
  focusedIndex: number
  isDark: boolean
  isCompact: boolean
  secondaryOpacity: number
  enableSmartActions: boolean
  enableUiPolish: boolean
  setItemRef: (index: number, element: HTMLDivElement | null) => void
}

/**
 * One virtualized row: positions a `HistoryItem` inside the window frame
 * the virtualizer computed via `style`. Rows past the data set (can happen
 * transiently while the list shrinks) render nothing.
 *
 * یک ردیف مجازی‌شده: `HistoryItem` را در قاب پنجره‌ای که مجازی‌ساز با
 * `style` محاسبه کرده قرار می‌دهد. ردیف‌های خارج از داده (که هنگام
 * کوچک‌شدن گذرا رخ می‌دهد) چیزی رسم نمی‌کنند.
 */
export function HistoryRow({ index, style, ...rowData }: { index: number; style: React.CSSProperties } & RowData) {
  const {
    items,
    setItemRef,
    onPaste,
    onDelete,
    onTogglePin,
    onFocus,
    focusedIndex,
    isDark,
    isCompact,
    secondaryOpacity,
    enableSmartActions,
    enableUiPolish,
  } = rowData

  if (index >= items.length) return null

  const item = items[index]
  return (
    <div style={style}>
      <HistoryItem
        ref={(el) => setItemRef(index, el)}
        item={item}
        index={index}
        isFocused={index === focusedIndex}
        onPaste={onPaste}
        onDelete={onDelete}
        onTogglePin={onTogglePin}
        onFocus={() => onFocus(index)}
        isDark={isDark}
        secondaryOpacity={secondaryOpacity}
        isCompact={isCompact}
        enableSmartActions={enableSmartActions}
        enableUiPolish={enableUiPolish}
      />
    </div>
  )
}
