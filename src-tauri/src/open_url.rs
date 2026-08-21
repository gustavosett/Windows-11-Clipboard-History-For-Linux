//! Safe URL opener used by Smart Actions.
//! Validation lives in Rust so the webview cannot bypass the TypeScript sanitizer.

use std::net::IpAddr;
use std::net::ToSocketAddrs;
use std::process::{Command, Stdio};
use url::Url;

use crate::net_policy::{is_disallowed_ip, looks_like_dotted_ipv4};

const MAX_URL_LEN: usize = 2048;

/// Open `raw` with `xdg-open` after an allowlist + DNS check. Never passes the
/// URL through a shell.
/// باز کردن `raw` با `xdg-open` پس از allowlist و بررسی DNS. هرگز URL از
/// طریق shell عبور نمی‌کند.
pub fn open_safe_url(raw: &str) -> Result<String, String> {
    let safe = validate_open_url(raw)?;

    // Post-DNS defence for https targets: a public-looking hostname may still
    // resolve to a private / metadata IP (DNS rebinding). The static check in
    // `validate_open_url` covers literal IPs only, so we resolve here before
    // handing the URL to the external browser.
    // دفاع پس از DNS برای مقاصد https: ممکن است یک نام عمومی به IP خصوصی یا
    // metadata رزولوشن شود. چک ایستای `validate_open_url` فقط IPهای صریح را
    // پوشش می‌دهد؛ بنابراین پیش از تحویل URL به مرورگر، رزولوشن می‌کنیم.
    let parsed = url::Url::parse(&safe).map_err(|e| format!("Invalid URL: {e}"))?;
    if parsed.scheme() == "https" {
        if let Some(host) = parsed.host_str() {
            let host = host.trim_matches(|c| c == '[' || c == ']').to_ascii_lowercase();
            if host.parse::<IpAddr>().is_err() && !looks_like_dotted_ipv4(&host) {
                resolve_public_host(&host)?;
            }
        }
    }

    Command::new("xdg-open")
        .arg(&safe)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to launch xdg-open: {e}"))?;
    Ok(safe)
}

pub fn validate_open_url(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_URL_LEN {
        return Err("URL is empty or too long".into());
    }
    if trimmed.chars().any(|c| c.is_control()) {
        return Err("URL contains control characters".into());
    }

    let parsed = Url::parse(trimmed).map_err(|e| format!("Invalid URL: {e}"))?;
    let scheme = parsed.scheme();
    // HTTPS-only for web targets: plain http:// is rejected so clipboard
    // content can never trigger a cleartext request. The frontend upgrades
    // http:// inputs to https:// before calling `open_safe_url`.
    // فقط HTTPS برای مقاصد وب: http:// ساده رد می‌شود تا محتوای کلیپ‌بورد
    // هرگز درخواست متنی‌آشکار (cleartext) ایجاد نکند. فرانت‌اند ورودی‌های
    // http:// را پیش از فراخوانی `open_safe_url` به https:// ارتقا می‌دهد.
    if scheme != "https" && scheme != "mailto" {
        return Err(format!("Protocol '{scheme}' is not allowed"));
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("URLs with credentials are not allowed".into());
    }
    if scheme == "mailto" {
        return Ok(parsed.to_string());
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| "URL is missing a host".to_string())?
        .trim_matches(|c| c == '[' || c == ']')
        .to_ascii_lowercase();

    if host == "localhost" || host.ends_with(".localhost") || host.ends_with(".internal") {
        return Err("Local/internal hosts are not allowed".into());
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_disallowed_ip(ip) {
            return Err("Private or reserved IP addresses are not allowed".into());
        }
    } else if looks_like_dotted_ipv4(&host) {
        return Err("Private or reserved IP addresses are not allowed".into());
    }

    Ok(parsed.to_string())
}

/// Resolve `host` and reject it unless every address is public.
/// رزولوشن `host` و رد آن مگر اینکه همهٔ آدرس‌ها عمومی باشند.
fn resolve_public_host(host: &str) -> Result<(), String> {
    let addrs: Vec<SocketAddr> = (host, 443)
        .to_socket_addrs()
        .map_err(|e| format!("DNS resolution failed for {host}: {e}"))?;
    if addrs.is_empty() {
        return Err(format!("No DNS records for {host}"));
    }
    for addr in &addrs {
        if is_disallowed_ip(addr.ip()) {
            return Err(format!(
                "Refusing to open {host}: it resolves to non-public IP {}",
                addr.ip()
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_public_https() {
        assert!(validate_open_url("https://example.com/docs").is_ok());
        assert!(validate_open_url("mailto:user@example.com").is_ok());
    }

    #[test]
    fn post_dns_rejects_local_resolving_host() {
        // `localhost` is statically allowed by hostname rules, but the
        // post-DNS check must refuse it because it resolves to 127.0.0.1.
        // `localhost` از نظر نام مجاز است اما چک پس از DNS باید آن را به‌دلیل
        // رزولوشن به 127.0.0.1 رد کند.
        assert!(resolve_public_host("localhost").is_err());
        // A definitely-unresolvable name must also fail closed.
        // نام قطعاً غیرقابل‌حل نیز باید fail-closed رد شود.
        assert!(resolve_public_host("definitely-not-a-real-host.invalid").is_err());
    }

    #[test]
    fn rejects_dangerous_targets() {
        assert!(validate_open_url("javascript:alert(1)").is_err());
        assert!(validate_open_url("http://example.com/").is_err());
        assert!(validate_open_url("http://127.0.0.1/").is_err());
        assert!(validate_open_url("https://localhost/admin").is_err());
        assert!(validate_open_url("https://169.254.169.254/latest").is_err());
        assert!(validate_open_url("https://user:pass@example.com/").is_err());
        assert!(validate_open_url("file:///etc/passwd").is_err());
    }
}
