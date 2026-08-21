/**
 * Playwright E2E Test Configuration
 * پیکربندی تست‌های E2E با Playwright
 * 
 * This configuration provides comprehensive end-to-end testing capabilities
 * for the clipboard history manager application.
 * این پیکربندی قابلیت‌های تست جامع end-to-end را برای برنامه فراهم می‌کند.
 * 
 * @author Mahdi Arts
 * @version 2.5.0
 */

import { defineConfig, devices, type PlaywrightTestConfig } from '@playwright/test';

/**
 * Application configuration for testing
 * پیکربندی برنامه برای تست
 */
const APP_CONFIG = {
  /** Application binary path relative to project root */
  binaryPath: '../src-tauri/target/release/windows-11-style-clipboard-history-manager-bin',
  /** Default launch arguments */
  args: ['--no-sandbox'],
  /** Environment variables for testing */
  env: {
    RUST_LOG: 'info',
    RUST_BACKTRACE: '1',
  },
};

/**
 * Timeouts for various operations (in milliseconds)
 * زمان‌های انتظار برای عملیات مختلف (به میلی‌ثانیه)
 */
const TIMEouts = {
  /** Default timeout for most operations */
  default: 30_000,
  /** Extended timeout for slow operations like window show */
  extended: 60_000,
  /** Timeout for file system operations */
  fileSystem: 10_000,
  /** Timeout for network operations */
  network: 15_000,
};

/**
 * Retry configuration for flaky tests
 * پیکربندی تلاش مجدد برای تست‌های ناپایدار
 */
const RETRY_CONFIG = {
  /** Number of retries for failed tests */
  failedTests: process.env.CI ? 2 : 0,
  /** Number of retries for flaky tests */
  flakyTests: 2,
};

/**
 * Main Playwright configuration
 * پیکربندی اصلی Playwright
 */
const config: PlaywrightTestConfig = defineConfig({
  /** Test directory pattern */
  testDir: './tests/e2e/playwright',
  
  /** Fully qualify test file names for better reporting */
  testMatch: '**/*.spec.ts',
  
  /** Timeout for each test */
  timeout: TIMEouts.default,
  
  /** Expect timeout */
  expect: {
    timeout: TIMEouts.default,
  },
  
  /** Run tests in parallel */
  fullyParallel: true,
  
  /** Fail the build on CI if there are test failures */
  forbidOnly: !!process.env.CI,
  
  /** Retry failed tests based on CI environment */
  retries: RETRY_CONFIG.failedTests,
  
  /** Workers to use for parallel execution */
  workers: process.env.CI ? 2 : undefined,
  
  /** Reporter configuration */
  reporter: [
    /** HTML report for local development */
    ['html', { 
      outputFolder: 'playwright-report',
      open: process.env.CI ? 'never' : 'on-failure',
    }],
    /** JSON report for CI integration */
    ['json', { 
      outputFile: 'test-results/playwright-results.json',
    }],
    /** List reporter for console output */
    ['list'],
  ],
  
  /** Shared settings for all tests */
  use: {
    /** Base URL for navigation */
    baseURL: process.env.TAURI_APP_URL || 'http://localhost:1420',
    
    /** Trace recording for debugging */
    trace: 'on-first-retry',
    
    /** Screenshot on failure */
    screenshot: 'only-on-failure',
    
    /** Video recording for debugging */
    video: process.env.CI ? 'retain-on-failure' : 'off',
    
    /** Artifacts to capture on failure */
    artifactsOnFailure: {
      /** Include trace files on failure */
      trace: true,
    },
    
    /** Default navigation timeout */
    navigationTimeout: TIMEouts.extended,
    
    /** Action timeout */
    actionTimeout: TIMEouts.default,
    
    /** Ignore HTTPS errors in development */
    ignoreHTTPSErrors: true,
    
    /** Viewport for tests */
    viewport: { width: 400, height: 600 },
    
    /** Launch options for all tests */
    launchOptions: {
      /** Don't open DevTools by default */
      devtools: false,
    },
  },
  
  /** Configure projects for different browsers/platforms */
  projects: [
    /** Primary test target: WebKit (Tauri's default renderer) */
    {
      name: 'webkit',
      use: {
        ...devices['Desktop Safari'],
        launchOptions: {
          args: ['--no-sandbox', '--disable-dev-shm-usage'],
        },
      },
    },
    /** Chromium as fallback for debugging */
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        launchOptions: {
          args: ['--no-sandbox', '--disable-dev-shm-usage'],
        },
      },
    },
    /** Firefox for cross-browser validation */
    {
      name: 'firefox',
      use: {
        ...devices['Desktop Firefox'],
        launchOptions: {
          args: ['--no-sandbox', '--disable-dev-shm-usage'],
        },
      },
    },
  ],
  
  /** Output directory for test artifacts */
  outputDir: 'test-results',
  
  /** Global teardown hook */
  globalTeardown: './tests/e2e/playwright/global-teardown.ts',
  
  /** Global setup hook */
  globalSetup: './tests/e2e/playwright/global-setup.ts',
});

export default config;

/**
 * Helper to export configuration for use in scripts
 * صادرکردن پیکربندی برای استفاده در اسکریپت‌ها
 */
export { APP_CONFIG, TIMEouts, RETRY_CONFIG };
