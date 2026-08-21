/**
 * Contract tests for the shared Settings-window SVG icons.
 * تست‌های قراردادی برای آیکون‌های SVG مشترک پنجرهٔ تنظیمات.
 *
 * Every icon must render a single accessible SVG that honours the `size`
 * and `className` props; CheckIcon intentionally overrides strokeWidth.
 * هر آیکون باید یک SVG دسترس‌پذیر بسازد که props ی `size` و `className`
 * را رعایت کند؛ CheckIcon عمداً strokeWidth را بازنویسی می‌کند.
 */
import { describe, expect, it } from 'vitest'
import { render } from '@testing-library/react'
import {
  AlertTriangleIcon,
  CheckIcon,
  KeyboardIcon,
  MonitorIcon,
  MoonIcon,
  PlusIcon,
  ResetIcon,
  SunIcon,
  TrashIcon,
  XIcon,
} from './icons'

const icons = [
  { name: 'MonitorIcon', Component: MonitorIcon, defaultSize: 24 },
  { name: 'MoonIcon', Component: MoonIcon, defaultSize: 24 },
  { name: 'SunIcon', Component: SunIcon, defaultSize: 24 },
  { name: 'ResetIcon', Component: ResetIcon, defaultSize: 16 },
  { name: 'KeyboardIcon', Component: KeyboardIcon, defaultSize: 24 },
  { name: 'TrashIcon', Component: TrashIcon, defaultSize: 24 },
  { name: 'AlertTriangleIcon', Component: AlertTriangleIcon, defaultSize: 20 },
  { name: 'CheckIcon', Component: CheckIcon, defaultSize: 16 },
  { name: 'PlusIcon', Component: PlusIcon, defaultSize: 16 },
  { name: 'XIcon', Component: XIcon, defaultSize: 14 },
] as const

describe('settings icons', () => {
  it.each(icons)(
    '$name uses its documented default size',
    ({ Component, defaultSize }) => {
      const { container } = render(<Component />)
      const svg = container.querySelector('svg')
      expect(svg).not.toBeNull()
      expect(svg).toHaveAttribute('width', String(defaultSize))
      expect(svg).toHaveAttribute('height', String(defaultSize))
    },
  )

  it.each(icons)('$name honours a custom size', ({ Component }) => {
    const { container } = render(<Component size={42} />)
    const svg = container.querySelector('svg')
    expect(svg).toHaveAttribute('width', '42')
    expect(svg).toHaveAttribute('height', '42')
  })

  it.each(icons)('$name forwards className to the svg element', ({ Component }) => {
    const { container } = render(<Component className="section-icon" />)
    expect(container.querySelector('svg')).toHaveClass('section-icon')
  })

  it.each(icons)('$name renders as a stroke-based outline icon', ({ Component }) => {
    const { container } = render(<Component />)
    const svg = container.querySelector('svg')
    // Shared visual contract: outline icons coloured via currentColor.
    // قرارداد بصری مشترک: آیکون‌های خطی با رنگ currentColor.
    expect(svg).toHaveAttribute('fill', 'none')
    expect(svg).toHaveAttribute('stroke', 'currentColor')
    expect(svg).toHaveAttribute('viewBox', '0 0 24 24')
  })

  it('CheckIcon uses a heavier stroke for emphasis', () => {
    const { container } = render(<CheckIcon />)
    expect(container.querySelector('svg')).toHaveAttribute('stroke-width', '2.5')
  })
})
