# ADR-0003: SSRF-hardened outbound downloads (DNS pinning)

- **Status:** Accepted (v2.0.0)
- **Date:** 2026-08-20

## Context

GIF search (Tenor) and GIF download are optional features. A classic SSRF
chain is: parse a URL → validate it → *later* connect to it. Between
validation and connection, DNS can be re-resolved to a private address
(DNS rebinding), turning the app into a proxy for internal services
(169.254.169.254 metadata endpoints, internal routers, …).

## Decision

Implement a three-layer validator (`ssrf.rs`):

1. **Parse + policy:** HTTPS only, no credentials in URL, host on the
   allowlist (`tenor.com`, `giphy.com`, `media.tenor.co`), direct-IP URLs
   rejected.
2. **Resolve + inspect:** all DNS answers are checked against a blocklist
   covering loopback, private, link-local, CGNAT, multicast, TEST-NET,
   documentation, and IPv4-mapped IPv6 ranges.
3. **Pin:** the HTTP client is built with `resolve_to_addrs(host, addrs)`
   so the connection *must* go to the already-validated addresses.
   Redirects are disabled (a new URL would have to be re-validated).

## Consequences

- The DNS-rebinding window between validation and connect is closed.
- Downloads are capped at 10 MB, streamed, with Content-Type sanity checks.
- The same validator is reused by the Tenor API proxy (`tenor_api.rs`) and
  the GIF downloader (`gif_manager.rs`).
