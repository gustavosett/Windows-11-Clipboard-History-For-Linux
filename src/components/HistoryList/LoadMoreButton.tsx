import { clsx } from 'clsx'
import { useTranslation } from 'react-i18next'

interface LoadMoreButtonProps {
  onClick: () => void
  isLoading: boolean
  isDark: boolean
}

/**
 * Pagination affordance under the virtualized list. Announced politely to
 * screen readers via the caller's `aria-live` region.
 *
 * دکمهٔ صفحه‌بندی زیر فهرست مجازی‌شده. از طریق ناحیهٔ `aria-live`
 * فراخواننده، محترمانه به صفحه‌خوان‌ها اعلام می‌شود.
 */
export function LoadMoreButton({ onClick, isLoading, isDark }: LoadMoreButtonProps) {
  const { t } = useTranslation()
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={isLoading}
      className={clsx(
        'text-[12px] font-medium px-4 py-1.5 rounded-full transition-all',
        'shadow-sm',
        isDark
          ? 'text-sky-100 bg-win11-bg-accent/80 hover:bg-win11-bg-accent'
          : 'text-white bg-win11-bg-accent hover:bg-[#006cbd]',
        'disabled:opacity-60 disabled:cursor-wait',
        'focus:outline-none focus-visible:ring-2 focus-visible:ring-win11-bg-accent focus-visible:ring-offset-2'
      )}
    >
      {isLoading ? t('common.loading') : t('clipboard.load_more')}
    </button>
  )
}
