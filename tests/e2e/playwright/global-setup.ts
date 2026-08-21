/**
 * Global Setup for Playwright E2E Tests
 * راه‌اندازی سراسری برای تست‌های E2E با Playwright
 * 
 * This file runs before all tests and ensures the testing environment
 * is properly configured.
 * این فایل قبل از همه تست‌ها اجرا می‌شود و محیط تست را پیکربندی می‌کند.
 * 
 * Tasks performed:
 * - Verify application binary exists
 * - Clean up any existing test artifacts
 * - Set up test data directories
 * - Verify system dependencies (Xvfb, WebKit)
 * 
 * عملیات انجام‌شده:
 * - تأیید وجود باینری برنامه
 * - پاک‌سازی آرتیفکت‌های تست موجود
 * - تنظیم دایرکتوری‌های داده تست
 * - تأیید وابستگی‌های سیستم (Xvfb، WebKit)
 */

import * as fs from 'fs';
import * as path from 'path';
import { execSync } from 'child_process';

/** Application data directories for testing */
const TEST_DATA_DIRS = [
  /** History database directory */
  '.local/share/windows-11-style-clipboard-history-manager',
  /** Configuration directory */
  '.config/windows-11-style-clipboard-history-manager',
  /** Cache directory */
  '.cache/windows-11-style-clipboard-history-manager',
  /** Log directory */
  '.local/share/windows-11-style-clipboard-history-manager/logs',
];

/** Test artifacts to clean up */
const TEST_ARTIFACTS = [
  /** Previous test results */
  'test-results',
  /** Previous HTML reports */
  'playwright-report',
  /** Test database files */
  'test-db-*',
];

/**
 * Verify that the application binary exists and is executable
 * تأیید وجود و قابلیت اجرای باینری برنامه
 */
function verifyBinary(): void {
  const binaryPath = path.resolve(
    process.cwd(),
    'src-tauri/target/release/windows-11-style-clipboard-history-manager-bin'
  );
  
  if (!fs.existsSync(binaryPath)) {
    /** Binary not found - this is expected in development without build */
    console.warn('[setup] Binary not found. Run `npm run tauri:build` first.');
    console.warn('[setup] E2E tests will be skipped.');
    process.env.SKIP_E2E = '1';
    return;
  }
  
  const stats = fs.statSync(binaryPath);
  if (!stats.mode) {
    throw new Error(`Binary ${binaryPath} is not executable`);
  }
  
  console.log(`[setup] Binary verified: ${binaryPath}`);
  process.env.SKIP_E2E = '0';
}

/**
 * Clean up test artifacts from previous runs
 * پاک‌سازی آرتیفکت‌های تست از اجراهای قبلی
 */
function cleanupArtifacts(): void {
  for (const artifact of TEST_ARTIFACTS) {
    const fullPath = path.resolve(process.cwd(), artifact);
    if (fs.existsSync(fullPath)) {
      if (fs.statSync(fullPath).isDirectory()) {
        fs.rmSync(fullPath, { recursive: true, force: true });
      } else {
        fs.unlinkSync(fullPath);
      }
      console.log(`[setup] Cleaned: ${artifact}`);
    }
  }
}

/**
 * Clean up test data directories
 * پاک‌سازی دایرکتوری‌های داده تست
 */
function cleanupTestData(): void {
  const homeDir = process.env.HOME || '/tmp';
  
  for (const dir of TEST_DATA_DIRS) {
    const fullPath = path.join(homeDir, dir);
    if (fs.existsSync(fullPath)) {
      fs.rmSync(fullPath, { recursive: true, force: true });
      console.log(`[setup] Cleaned test data: ${dir}`);
    }
  }
}

/**
 * Verify system dependencies
 * تأیید وابستگی‌های سیستم
 */
function verifySystemDeps(): void {
  try {
    /** Check for Xvfb (X Virtual Framebuffer) */
    execSync('which Xvfb', { stdio: 'pipe' });
    console.log('[setup] Xvfb found');
  } catch {
    console.warn('[setup] Xvfb not found - some tests may fail');
  }
  
  try {
    /** Check for WebKit dependencies */
    execSync('ldconfig -p | grep webkit2gtk', { stdio: 'pipe' });
    console.log('[setup] WebKit2GTK found');
  } catch {
    console.warn('[setup] WebKit2GTK not found - some tests may fail');
  }
}

/**
 * Set up environment variables for testing
 * تنظیم متغیرهای محیطی برای تست
 */
function setupEnvironment(): void {
  /** Disable GPU acceleration for headless testing */
  process.env.ELECTRON_OVERRIDE_DIST_PATH = '1';
  process.env.GTKWIDGET_SIMPLE_HEADLESS = '1';
  
  /** Set test mode */
  process.env.TESTING_MODE = '1';
  
  /** Disable animations for faster tests */
  process.env.CSS_ANIMATIONS = 'disabled';
  
  console.log('[setup] Environment configured');
}

/**
 * Create necessary directories
 * ایجاد دایرکتوری‌های لازم
 */
function createDirectories(): void {
  const dirs = [
    'test-results',
    'playwright-report',
    'test-data',
  ];
  
  for (const dir of dirs) {
    const fullPath = path.resolve(process.cwd(), dir);
    if (!fs.existsSync(fullPath)) {
      fs.mkdirSync(fullPath, { recursive: true });
      console.log(`[setup] Created directory: ${dir}`);
    }
  }
}

/**
 * Main setup function
 * تابع اصلی راه‌اندازی
 */
/**
 * Playwright runs this as a synchronous setup hook — no work here is
 * asynchronous, so the function is intentionally not `async`.
 * Playwright این را به‌عنوان هوک setup هم‌زمان اجرا می‌کند؛ هیچ عملیاتی در
 * این‌جا ناهمگام نیست، بنابراین تابع عمداً `async` نیست.
 */
function globalSetup(): void {
  console.log('='.repeat(60));
  console.log('[setup] Starting E2E test environment setup...');
  console.log('='.repeat(60));
  
  /** Step 1: Verify binary */
  verifyBinary();
  
  /** Step 2: Clean up artifacts */
  cleanupArtifacts();
  
  /** Step 3: Set up environment */
  setupEnvironment();
  
  /** Step 4: Create directories */
  createDirectories();
  
  /** Step 5: Verify system deps */
  verifySystemDeps();
  
  /** Step 6: Clean test data */
  cleanupTestData();
  
  console.log('='.repeat(60));
  console.log('[setup] Environment setup complete!');
  console.log('[setup] Ready to run E2E tests.');
  console.log('='.repeat(60));
}

export default globalSetup;
