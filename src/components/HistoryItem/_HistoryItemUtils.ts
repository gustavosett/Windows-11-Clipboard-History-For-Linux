import { clsx } from 'clsx'

export function getIconSize(effectiveCompact: boolean) {
  return effectiveCompact ? 'w-3 h-3' : 'w-4 h-4'
}

export function getIconContainerClasses(effectiveCompact: boolean) {
  return clsx(
    'flex-shrink-0 rounded-md flex items-center justify-center',
    effectiveCompact ? 'w-6 h-6' : 'w-8 h-8'
  )
}
