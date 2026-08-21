/**
 * History list building blocks — extracted from `ClipboardTab` so the tab
 * stays focused on state and orchestration while rendering concerns live
 * next to their single use.
 *
 * اجزای سازندهٔ فهرست تاریخچه — از `ClipboardTab` جدا شده‌اند تا تب روی
 * وضعیت و هماهنگی متمرکز بماند و مسائل رندر کنار کاربرد واحدشان زندگی
 * کنند.
 */
export { HistoryRow, type RowData } from './HistoryRow'
export { PinnedSection, RecentSectionLabel, type HistoryItemVisualProps } from './PinnedSection'
export { LoadMoreButton } from './LoadMoreButton'
