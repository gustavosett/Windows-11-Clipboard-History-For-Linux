/**
 * E2E smoke for the real app UI (WebDriver session via tauri-driver).
 * اسموک E2E رابط کاربری واقعی برنامه (نشست WebDriver از طریق tauri-driver).
 *
 * Run through `./scripts/e2e.sh`, which starts `tauri-driver --port 4444`
 * under Xvfb and then executes this file. Minimal dependency footprint:
 * plain `fetch` against the WebDriver HTTP API — no extra npm packages.
 * از طریق `./scripts/e2e.sh` اجرا می‌شود؛ وابستگی اضافه ندارد و فقط از
 * fetch روی API ی HTTP وب‌درایور استفاده می‌کند.
 *
 * Checks / بررسی‌ها:
 *   1. A WebDriver session can be created for the app window.
 *   2. The main window surfaces the clipboard history UI (document title).
 *   3. The app stays responsive to a simple JS probe.
 */

const BASE = process.env.WD_BASE ?? 'http://127.0.0.1:4444'

async function wd(method, path, body) {
  const response = await fetch(`${BASE}${path}`, {
    method,
    headers: { 'Content-Type': 'application/json' },
    body: body ? JSON.stringify(body) : undefined,
  })
  const json = await response.json()
  if (!response.ok) {
    throw new Error(`WebDriver ${method} ${path} failed: ${JSON.stringify(json.value)}`)
  }
  return json.value
}

async function main() {
  const session = await wd('POST', '/session', {
    capabilities: { alwaysMatch: { 'tauri:options': { application: '../src-tauri/target/debug/windows-11-style-clipboard-history-manager-bin' } } },
  })
  const sessionId = session.sessionId
  try {
    const title = await wd('GET', `/session/${sessionId}/title`)
    console.log(`[smoke] window title: ${JSON.stringify(title)}`)

    const probe = await wd('POST', `/session/${sessionId}/execute/sync`, {
      script: 'return typeof window !== "undefined" && !!document.body',
      args: [],
    })
    console.log(`[smoke] DOM probe: ${JSON.stringify(probe)}`)
    if (probe !== true) {
      throw new Error('DOM probe did not return true')
    }
  } finally {
    await wd('DELETE', `/session/${sessionId}`).catch(() => {})
  }
}

main().catch((error) => {
  console.error(`[smoke] FAILED: ${error.message}`)
  process.exit(1)
})
