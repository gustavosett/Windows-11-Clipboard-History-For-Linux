import { clsx } from 'clsx'

export function Switch({
  checked,
  onChange,
  isDark,
}: {
  checked: boolean
  onChange: (v: boolean) => void
  isDark: boolean
}) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className={clsx(
        'relative inline-flex h-6 w-11 shrink-0 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-win11-bg-accent focus:ring-offset-2',
        checked ? 'bg-win11-bg-accent' : isDark ? 'bg-white/10' : 'bg-gray-300',
        isDark ? 'focus:ring-offset-gray-900' : 'focus:ring-offset-white'
      )}
    >
      <span
        className={clsx(
          'inline-block h-4 w-4 transform rounded-full bg-white transition-transform',
          checked ? 'translate-x-6' : 'translate-x-1'
        )}
      />
    </button>
  )
}
