import { clsx } from 'clsx'
import type { ClipboardItem } from '../../types/clipboard'

export function TextContent({
  item,
  isDark,
  effectiveCompact,
}: {
  item: ClipboardItem
  isDark: boolean
  effectiveCompact: boolean
}) {
  if (item.content.type !== 'Text') return null
  return (
    <p
      className={clsx(
        'text-sm break-words whitespace-pre-wrap',
        effectiveCompact ? 'line-clamp-1' : 'line-clamp-3',
        isDark ? 'text-win11-text-primary' : 'text-win11Light-text-primary'
      )}
    >
      {item.content.data}
    </p>
  )
}

export function ImageContent({
  item,
  isDark,
  effectiveCompact,
}: {
  item: ClipboardItem
  isDark: boolean
  effectiveCompact: boolean
}) {
  if (item.content.type !== 'Image') return null
  const { width, height, base64 } = item.content.data

  if (effectiveCompact) {
    return (
      <span
        className={clsx(
          'text-sm italic',
          isDark ? 'text-win11-text-tertiary' : 'text-win11Light-text-secondary'
        )}
      >
        Image ({width}×{height})
      </span>
    )
  }

  return (
    <div className="relative">
      <img
        src={`data:image/png;base64,${base64}`}
        alt="Clipboard image"
        className="max-w-full max-h-24 rounded object-contain bg-black/10"
      />
      <span className="absolute bottom-1 right-1 text-xs px-1.5 py-0.5 rounded bg-black/60 text-white">
        {width}×{height}
      </span>
    </div>
  )
}

export function Timestamp({
  show,
  isDark,
  timestamp,
}: {
  show: boolean
  isDark: boolean
  timestamp: string
}) {
  if (!show) return null

  const formatTime = (timestamp: string) => {
    const date = new Date(timestamp)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffMins = Math.floor(diffMs / 60000)
    const diffHours = Math.floor(diffMs / 3600000)

    if (diffMins < 1) return 'Just now'
    if (diffMins < 60) return `${diffMins}m ago`
    if (diffHours < 24) return `${diffHours}h ago`
    return date.toLocaleDateString()
  }

  return (
    <span
      className={clsx(
        'text-xs mt-1 block',
        isDark ? 'text-win11-text-tertiary' : 'text-win11Light-text-secondary'
      )}
    >
      {formatTime(timestamp)}
    </span>
  )
}
