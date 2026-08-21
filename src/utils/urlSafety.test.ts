import { describe, it, expect } from 'vitest'
import { sanitizeOpenUrl, normalizeHttpUrl } from './urlSafety'

describe('sanitizeOpenUrl', () => {
  it('accepts valid https URLs', () => {
    expect(sanitizeOpenUrl('https://example.com')).toBe('https://example.com/')
  })

  it('rejects plain http URLs (HTTPS-only policy)', () => {
    expect(sanitizeOpenUrl('http://example.com')).toBeNull()
    expect(sanitizeOpenUrl('http://www.example.org/path')).toBeNull()
  })

  it('accepts mailto URLs', () => {
    expect(sanitizeOpenUrl('mailto:user@example.com')).toBe('mailto:user@example.com')
  })

  it('rejects javascript: URLs', () => {
    expect(sanitizeOpenUrl('javascript:alert(1)')).toBeNull()
  })

  it('rejects file: URLs', () => {
    expect(sanitizeOpenUrl('file:///etc/passwd')).toBeNull()
  })

  it('rejects data: URLs', () => {
    expect(sanitizeOpenUrl('data:text/html,<h1>hi</h1>')).toBeNull()
  })

  it('rejects URLs with credentials', () => {
    expect(sanitizeOpenUrl('https://user:pass@example.com')).toBeNull()
  })

  it('rejects localhost URLs', () => {
    expect(sanitizeOpenUrl('http://localhost:3000')).toBeNull()
  })

  it('rejects loopback and metadata IPs', () => {
    expect(sanitizeOpenUrl('http://127.0.0.1/')).toBeNull()
    expect(sanitizeOpenUrl('http://169.254.169.254/latest')).toBeNull()
  })

  it('rejects *.localhost URLs', () => {
    expect(sanitizeOpenUrl('http://api.localhost:8080')).toBeNull()
  })

  it('rejects empty strings', () => {
    expect(sanitizeOpenUrl('')).toBeNull()
  })

  it('rejects whitespace-only strings', () => {
    expect(sanitizeOpenUrl('   ')).toBeNull()
  })

  it('rejects URLs exceeding 2048 characters', () => {
    const longUrl = 'https://example.com/' + 'a'.repeat(2050)
    expect(sanitizeOpenUrl(longUrl)).toBeNull()
  })

  it('trims whitespace', () => {
    expect(sanitizeOpenUrl('  https://example.com  ')).toBe('https://example.com/')
  })
})

describe('normalizeHttpUrl', () => {
  it('returns https URLs unchanged', () => {
    expect(normalizeHttpUrl('https://example.com')).toBe('https://example.com')
  })

  it('upgrades http URLs to https (HTTPS-only policy)', () => {
    expect(normalizeHttpUrl('http://example.com')).toBe('https://example.com')
  })

  it('prepends https:// to bare domains', () => {
    expect(normalizeHttpUrl('example.com')).toBe('https://example.com')
  })

  it('trims whitespace before normalizing', () => {
    expect(normalizeHttpUrl('  example.com  ')).toBe('https://example.com')
  })
})

describe('urlSafety edge cases (SSRF hardening)', () => {
  it('rejects obfuscated IPv4 forms', () => {
    // Decimal / hex / octal encodings of 127.0.0.1 and friends
    expect(sanitizeOpenUrl('http://2130706433/')).toBeNull()
    expect(sanitizeOpenUrl('http://0x7f000001/')).toBeNull()
    expect(sanitizeOpenUrl('http://0177.0.0.1/')).toBeNull()
  })

  it('rejects IPv6 loopback, mapped, link-local, ULA and documentation ranges', () => {
    expect(sanitizeOpenUrl('http://[::1]/')).toBeNull()
    expect(sanitizeOpenUrl('http://[::]/')).toBeNull()
    expect(sanitizeOpenUrl('http://[::ffff:127.0.0.1]/')).toBeNull()
    expect(sanitizeOpenUrl('http://[2001:db8::1]/')).toBeNull()
    expect(sanitizeOpenUrl('http://[fe80::1]/')).toBeNull()
    expect(sanitizeOpenUrl('http://[fc00::1]/')).toBeNull()
    expect(sanitizeOpenUrl('http://[fd12:3456::1]/')).toBeNull()
  })

  it('rejects private, CGNAT, benchmark and multicast IPv4 ranges', () => {
    expect(sanitizeOpenUrl('http://10.0.0.5/')).toBeNull()
    expect(sanitizeOpenUrl('http://172.16.0.1/')).toBeNull()
    expect(sanitizeOpenUrl('http://192.168.1.1/')).toBeNull()
    expect(sanitizeOpenUrl('http://100.64.0.1/')).toBeNull()
    expect(sanitizeOpenUrl('http://198.18.0.1/')).toBeNull()
    expect(sanitizeOpenUrl('http://224.0.0.1/')).toBeNull()
    expect(sanitizeOpenUrl('http://0.0.0.0/')).toBeNull()
  })

  it('still allows public IPv4 and global IPv6 addresses over https', () => {
    expect(sanitizeOpenUrl('https://8.8.8.8/dns')).not.toBeNull()
    expect(sanitizeOpenUrl('https://[2606:4700::1111]/')).not.toBeNull()
    expect(sanitizeOpenUrl('https://[2001:4860:4860::8888]/')).not.toBeNull()
  })

  it('rejects .internal and metadata hostnames', () => {
    expect(sanitizeOpenUrl('http://metadata.google.internal/')).toBeNull()
    expect(sanitizeOpenUrl('http://instance-data.internal/')).toBeNull()
    expect(sanitizeOpenUrl('https://myhost.internal/')).toBeNull()
  })

  it('rejects control characters and overlong URLs', () => {
    expect(sanitizeOpenUrl('https://example.com/\u0000')).toBeNull()
    expect(sanitizeOpenUrl('https://example.com/' + 'a'.repeat(2100))).toBeNull()
  })

  it('rejects whitespace-padded javascript: URLs', () => {
    expect(sanitizeOpenUrl(' javascript:alert(1)')).toBeNull()
  })

  it('treats backslash in special schemes as a path separator (WHATWG)', () => {
    // https://example.com\@evil.com parses as https://example.com/@evil.com
    const safe = sanitizeOpenUrl('https://example.com\\@evil.com')
    expect(safe).not.toBeNull()
    expect(new URL(safe as string).hostname).toBe('example.com')
  })

  it('normalizes bare domains to https', () => {
    expect(normalizeHttpUrl('example.com')).toBe('https://example.com')
    expect(normalizeHttpUrl('https://example.com')).toBe('https://example.com')
    expect(normalizeHttpUrl('http://example.com')).toBe('https://example.com')
  })

  it('accepts legitimate public URLs', () => {
    expect(sanitizeOpenUrl('https://example.com/path?q=1#frag')).not.toBeNull()
    expect(sanitizeOpenUrl('https://sub.example.co.uk:8443/x')).not.toBeNull()
    expect(sanitizeOpenUrl('mailto:user@example.com')).not.toBeNull()
  })
})
