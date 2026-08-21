// Vitest global setup: registers jest-dom matchers (toBeInTheDocument, etc.)
import '@testing-library/jest-dom/vitest'
import '../i18n/config'
import { cleanup } from '@testing-library/react'
import { afterEach } from 'vitest'

// React Testing Library auto-cleanup relies on a global afterEach; wire it
// explicitly so rendered DOM cannot leak between tests.
afterEach(() => {
  cleanup()
})

// jsdom does not implement scrollIntoView (used by CategoryStrip keyboard nav)
if (!Element.prototype.scrollIntoView) {
  Element.prototype.scrollIntoView = () => {}
}
