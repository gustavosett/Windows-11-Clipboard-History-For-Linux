/// <reference types="vitest/config" />
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      // Coverage gates target the core logic + presentational components that
      // have dedicated unit tests. Keeping the set explicit makes the gate
      // meaningful: a regression in these files must push coverage down.
      // Large window-shell components are exercised by the CI smoke test.
      include: [
        'src/utils/historySearch.ts',
        'src/utils/urlSafety.ts',
        'src/utils/pagination.ts',
        'src/services/smartActionService.ts',
        'src/hooks/useClipboardHistory.ts',
        'src/components/EmptyState.tsx',
        'src/components/Switch.tsx',
        'src/components/Header.tsx',
        'src/components/common/SearchBar.tsx',
        'src/components/common/CategoryStrip.tsx',
        'src/components/settings/KeyboardShortcutsSection.tsx',
        'src/components/settings/icons.tsx',
        'src/components/settings/SectionCard.tsx',
      ],
      exclude: ['src/**/*.test.{ts,tsx}'],
      reporter: ['text', 'json-summary'],
      thresholds: {
        lines: 75,
        functions: 65,
        branches: 60,
        statements: 75,
      },
    },
  },

  // Tauri expects a fixed port for development
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // Use polling to keep file watching reliable across Linux environments.
      usePolling: true,
    },
  },

  // Clear screen during dev
  clearScreen: false,

  // Environment variables prefix
  envPrefix: ['VITE_', 'TAURI_'],

  build: {
    // Linux builds run on WebKitGTK.
    target: 'safari13',
    // Don't minify for debug builds
    minify: process.env.TAURI_DEBUG ? false : 'esbuild',
    // Produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
    rollupOptions: {
      input: {
        main: 'index.html',
      },
    },
  },
})
