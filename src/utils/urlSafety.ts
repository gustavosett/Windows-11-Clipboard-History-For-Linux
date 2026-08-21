// HTTPS-only for web targets: plain http:// is rejected so clipboard content
// can never trigger a cleartext request. The Rust `open_safe_url` command
// re-validates the same policy (see src-tauri/src/open_url.rs).
// فقط HTTPS برای مقاصد وب: http:// ساده رد می‌شود تا محتوای کلیپ‌بورد هرگز
// درخواست متنی‌آشکار ایجاد نکند. فرمان Rust ی `open_safe_url` همان سیاست را
// دوباره اعتبارسنجی می‌کند (src-tauri/src/open_url.rs را ببینید).
const ALLOWED_PROTOCOLS = new Set(['https:', 'mailto:'])

/**
 * True when `s` contains raw control characters (incl. NUL). These must
 * never reach a URL parser or the shell opener. Implemented as a scan
 * instead of a regex so linters treat it as intentional.
 */
function hasControlChars(s: string): boolean {
  for (let i = 0; i < s.length; i++) {
    const code = s.charCodeAt(i)
    if (code < 0x20 || code === 0x7f) return true
  }
  return false
}

function isDisallowedIpv4(host: string): boolean {
  const parts = host.split('.')
  if (parts.length !== 4) return false
  const octets = parts.map((p) => Number(p))
  if (octets.some((o) => !Number.isInteger(o) || o < 0 || o > 255)) return false
  const [a, b] = octets

  return (
    // 0.0.0.0/8, 10.0.0.0/8, 127.0.0.0/8, 169.254.0.0/16 (incl. cloud metadata)
    a === 0 ||
    a === 10 ||
    a === 127 ||
    a === 169 ||
    // 100.64.0.0/10 CGNAT
    (a === 100 && b >= 64 && b <= 127) ||
    // 172.16.0.0/12 private
    (a === 172 && b >= 16 && b <= 31) ||
    // 192.168.0.0/16 private
    (a === 192 && b === 168) ||
    // 192.0.0.0/24 IETF, 192.0.2.0/24 TEST-NET-1
    (a === 192 && (b === 0 || b === 2)) ||
    // 198.18.0.0/15 benchmark, 198.51.100.0/24 TEST-NET-2
    (a === 198 && (b === 18 || b === 19 || b === 51)) ||
    // 203.0.113.0/24 TEST-NET-3
    (a === 203 && b === 0) ||
    // 224.0.0.0/4 multicast + reserved, 255.255.255.255 broadcast
    a >= 224 ||
    a === 255
  )
}

function isDisallowedIpv6(host: string): boolean {
  // WHATWG URL normalizes IPv6 to its canonical lowercase form (with zone id).
  const h = host.toLowerCase().split('%')[0]

  // Unspecified, loopback, IPv4-mapped / translated
  if (h === '::' || h === '::1' || h.startsWith('::ffff:')) return true
  if (h === '0:0:0:0:0:0:0:0' || h === '0:0:0:0:0:0:0:1') return true

  const first = h.split(':')[0]
  // 0000::/8 reserved
  if (first === '0' || first === '0000') return true
  // fc00::/7 unique-local
  if (first.startsWith('fc') || first.startsWith('fd')) return true
  // fe80::/10 link-local
  if (
    first.startsWith('fe8') ||
    first.startsWith('fe9') ||
    first.startsWith('fea') ||
    first.startsWith('feb')
  )
    return true
  // 2001:db8::/32 documentation
  if (h.startsWith('2001:db8')) return true

  return false
}

export function sanitizeOpenUrl(raw: string): string | null {
  const trimmed = raw.trim()
  if (!trimmed || trimmed.length > 2048) return null
  if (hasControlChars(trimmed)) return null

  try {
    const url = new URL(trimmed)
    if (!ALLOWED_PROTOCOLS.has(url.protocol)) return null
    if (url.username || url.password) return null
    if (url.protocol === 'mailto:') return url.toString()

    // WHATWG URL keeps IPv6 brackets in hostname; strip them for checks.
    const hostname = url.hostname.replace(/^\[|\]$/g, '').toLowerCase()

    if (hostname === 'localhost' || hostname.endsWith('.localhost')) return null
    if (hostname === '169.254.169.254' || hostname.endsWith('.internal')) return null

    if (hostname.includes(':')) {
      if (isDisallowedIpv6(hostname)) return null
    } else if (isDisallowedIpv4(hostname)) {
      return null
    }

    return url.toString()
  } catch {
    return null
  }
}

/**
 * Normalize user input into an openable URL.
 *
 * Bare domains get the `https://` prefix; explicit `http://` input is
 * upgraded to `https://` (HTTPS-only policy). Sites that do not serve
 * HTTPS will simply fail to open — the security trade-off is deliberate.
 *
 * نرمال‌سازی ورودی کاربر به یک URL قابل باز شدن.
 *
 * دامنه‌های بدون پروتکل پیشوند `https://` می‌گیرند؛ ورودی صریح `http://`
 * هم به `https://` ارتقا می‌یابد (سیاست فقط-HTTPS). سایت‌هایی که HTTPS
 * ارائه نمی‌دهند باز نمی‌شوند — این مصالحهٔ امنیتی آگاهانه است.
 */
export function normalizeHttpUrl(raw: string): string {
  const trimmed = raw.trim()
  if (/^https:\/\//i.test(trimmed)) return trimmed
  if (/^http:\/\//i.test(trimmed)) return trimmed.replace(/^http:/i, 'https:')
  return `https://${trimmed}`
}
