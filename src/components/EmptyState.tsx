import { ClipboardList } from 'lucide-react'

/**
 * Empty state component when there's no clipboard history
 */
export function EmptyState() {
  return (
    <div
      className="flex flex-col items-center justify-center h-full py-12 px-4 text-center"
      data-tauri-drag-region
      tabIndex={-1}
    >
      <div className="w-16 h-16 rounded-full dark:bg-win11-bg-tertiary bg-win11Light-bg-tertiary flex items-center justify-center mb-4">
        <ClipboardList className="w-8 h-8 dark:text-win11-text-tertiary text-win11Light-text-secondary" />
      </div>

      <h3 className="text-base font-medium dark:text-win11-text-primary text-win11Light-text-primary mb-2">
        No clipboard history yet
      </h3>

      <p className="text-sm dark:text-win11-text-secondary text-win11Light-text-secondary max-w-[200px]">
        Copy something to see it appear here. Press{' '}
        <kbd className="px-1.5 py-0.5 rounded dark:bg-win11-bg-tertiary bg-win11Light-bg-tertiary text-xs font-mono">
          Super+V
        </kbd>{' '}
        to open anytime.
      </p>
    </div>
  )
}
