import { clsx } from 'clsx'
import { ReactNode } from 'react'

interface PickerLayoutProps {
  header: ReactNode
  subHeader?: ReactNode
  footer: ReactNode
  children: ReactNode
}

export function PickerLayout({ header, subHeader, footer, children }: PickerLayoutProps) {
  return (
    <div className="flex flex-col h-full overflow-hidden select-none">
      {/* Search / Header */}
      <div className="px-3 pt-3 pb-2 flex-shrink-0">{header}</div>

      {/* Categories / Recents */}
      {subHeader && <div className="px-3 pb-2 flex-shrink-0">{subHeader}</div>}

      {/* Grid Container */}
      <div className="flex-1 min-h-0 overflow-hidden relative">{children}</div>

      {/* Footer / Status Bar */}
      <div
        className={clsx(
          'px-3 py-2 h-10 flex-shrink-0',
          'border-t dark:border-win11-border-subtle border-win11Light-border',
          'flex items-center gap-2'
        )}
      >
        {footer}
      </div>
    </div>
  )
}
