/**
 * Global Teardown for Playwright E2E Tests
 * خاتمه سراسری برای تست‌های E2E با Playwright
 * 
 * This file runs after all tests complete and performs cleanup tasks.
 * این فایل پس از اتمام همه تست‌ها اجرا می‌شود و عملیات پاک‌سازی را انجام می‌دهد.
 * 
 * Tasks performed:
 * - Clean up test data directories
 * - Archive test results for CI
 * - Generate summary report
 * 
 * عملیات انجام‌شده:
 * - پاک‌سازی دایرکتوری‌های داده تست
 * - آرشیو نتایج تست برای CI
 * - تولید گزارش خلاصه
 */

import * as fs from 'fs';
import * as path from 'path';
import { execSync } from 'child_process';

/**
 * Shape of the subset of Playwright's JSON report we summarise.
 * گِردِ subset گزارش JSON پلی‌رایت که خلاصه می‌کنیم.
 */
interface PlaywrightReportStats {
  tests?: number;
  passed?: number;
  failed?: number;
  skipped?: number;
  duration?: number;
}

interface PlaywrightJsonReport {
  stats?: PlaywrightReportStats;
}

/** Best-effort human-readable description of an unknown thrown value. */
/** توصیف خوانا و best-effort از یک مقدار پرتاب‌شدهٔ ناشناخته. */
function describeError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/** Application data directories to clean */
const TEST_DATA_DIRS = [
  '.local/share/windows-11-style-clipboard-history-manager',
  '.config/windows-11-style-clipboard-history-manager',
  '.cache/windows-11-style-clipboard-history-manager',
];

/**
 * Clean up test data directories
 * پاک‌سازی دایرکتوری‌های داده تست
 */
function cleanupTestData(): void {
  const homeDir = process.env.HOME || '/tmp';
  
  for (const dir of TEST_DATA_DIRS) {
    const fullPath = path.join(homeDir, dir);
    if (fs.existsSync(fullPath)) {
      try {
        fs.rmSync(fullPath, { recursive: true, force: true });
        console.log(`[teardown] Cleaned: ${dir}`);
      } catch (error) {
        console.warn(`[teardown] Failed to clean ${dir}: ${describeError(error)}`);
      }
    }
  }
}

/**
 * Archive test results for CI
 * آرشیو نتایج تست برای CI
 */
function archiveTestResults(): void {
  if (!process.env.CI) {
    return; /** Only archive in CI environment */
  }
  
  try {
    const resultsDir = path.resolve(process.cwd(), 'test-results');
    if (fs.existsSync(resultsDir)) {
      /** Results will be automatically uploaded by CI provider */
      console.log('[teardown] Test results available for upload');
    }
  } catch (error) {
    console.warn(`[teardown] Failed to archive results: ${describeError(error)}`);
  }
}

/**
 * Generate summary report
 * تولید گزارش خلاصه
 */
function generateSummary(): void {
  const resultsFile = path.resolve(process.cwd(), 'test-results/playwright-results.json');
  
  if (!fs.existsSync(resultsFile)) {
    console.log('[teardown] No test results to summarize');
    return;
  }
  
  try {
    // Parsed JSON is `unknown`; narrow it before reading `.stats` so the
    // type-aware lint can verify every access is safe.
    // JSON تجزیه‌شده از نوع `unknown` است؛ پیش از خواندن `.stats` آن را
    // محدود می‌کنیم تا لینت آگاه به نوع، امنیت همهٔ دسترسی‌ها را تأیید کند.
    const results: PlaywrightJsonReport = JSON.parse(
      fs.readFileSync(resultsFile, 'utf-8')
    ) as PlaywrightJsonReport;
    const s = results.stats ?? {};
    
    const stats = {
      total: s.tests ?? 0,
      passed: s.passed ?? 0,
      failed: s.failed ?? 0,
      skipped: s.skipped ?? 0,
      duration: s.duration ?? 0,
    };
    
    console.log('\n' + '='.repeat(60));
    console.log('[teardown] Test Summary:');
    console.log('='.repeat(60));
    console.log(`  Total:  ${stats.total}`);
    console.log(`  Passed: ${stats.passed} ${stats.total > 0 ? `(${(stats.passed / stats.total * 100).toFixed(1)}%)` : ''}`);
    console.log(`  Failed: ${stats.failed}`);
    console.log(`  Skipped: ${stats.skipped}`);
    console.log(`  Duration: ${(stats.duration / 1000).toFixed(2)}s`);
    console.log('='.repeat(60) + '\n');
  } catch (error) {
    console.warn(`[teardown] Failed to generate summary: ${describeError(error)}`);
  }
}

/**
 * Kill any remaining processes
 * پایان دادن به فرآیندهای باقی‌مانده
 */
function killRemainingProcesses(): void {
  try {
    /** Kill any lingering app processes */
    execSync(
      'pkill -f "windows-11-style-clipboard-history-manager" 2>/dev/null || true',
      { stdio: 'pipe' }
    );
    console.log('[teardown] Cleaned up lingering processes');
  } catch {
    /** Process cleanup is best-effort */
  }
}

/**
 * Main teardown function
 * تابع اصلی خاتمه
 */
/**
 * Playwright runs this as a synchronous teardown hook; nothing here is
 * asynchronous, so the function is intentionally not `async`.
 * Playwright این را به‌عنوان هوک خاتمه هم‌زمان اجرا می‌کند؛ هیچ عملیاتی در
 * این‌جا ناهمگام نیست، بنابراین تابع عمداً `async` نیست.
 */
function globalTeardown(): void {
  console.log('='.repeat(60));
  console.log('[teardown] Starting E2E test cleanup...');
  console.log('='.repeat(60));
  
  /** Step 1: Generate summary */
  generateSummary();
  
  /** Step 2: Archive results */
  archiveTestResults();
  
  /** Step 3: Kill remaining processes */
  killRemainingProcesses();
  
  /** Step 4: Clean test data */
  cleanupTestData();
  
  console.log('='.repeat(60));
  console.log('[teardown] Cleanup complete!');
  console.log('='.repeat(60));
}

export default globalTeardown;
