import { ClipboardList } from 'lucide-react'
import { clsx } from 'clsx'

interface EmptyStateProps {
  isDark: boolean
}

/**
 * Empty state component when there's no clipboard history
 */
export function EmptyState({ isDark }: EmptyStateProps) {
  return (
    <div
      className="flex flex-col items-center justify-center h-full py-12 px-4 text-center"
      data-tauri-drag-region
    >
      <div
        className={clsx(
          'w-16 h-16 rounded-full flex items-center justify-center mb-4',
          isDark ? 'bg-win11-bg-tertiary' : 'bg-win11Light-bg-tertiary'
        )}
      >
        <ClipboardList
          className={clsx(
            'w-8 h-8',
            isDark ? 'text-win11-text-tertiary' : 'text-win11Light-text-secondary'
          )}
        />
      </div>

      <h3
        className={clsx(
          'text-base font-medium mb-2',
          isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
        )}
      >
        No clipboard history yet
      </h3>

      <p
        className={clsx(
          'text-sm max-w-[200px]',
          isDark ? 'text-win11-text-secondary' : 'text-win11Light-text-secondary'
        )}
      >
        Copy something to see it appear here. Press{' '}
        <kbd
          className={clsx(
            'px-1.5 py-0.5 rounded text-xs font-mono',
            isDark ? 'bg-win11-bg-tertiary' : 'bg-win11Light-bg-tertiary'
          )}
        >
          Super+V
        </kbd>{' '}
        to open anytime.
      </p>
    </div>
  )
}
