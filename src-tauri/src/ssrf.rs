//! SSRF protection for outbound downloads (GIF paste).
//! HTTPS-only, host allowlist, DNS resolution checks, no redirects.

use std::net::SocketAddr;
use std::net::ToSocketAddrs;
#[cfg(feature = "gif-search")]
use std::time::Duration;
use url::Url;

use crate::net_policy::{is_disallowed_ip, looks_like_dotted_ipv4};

/// A URL that has been allowlisted and whose DNS records were all public.
#[derive(Debug, Clone)]
pub struct ValidatedDownload {
    pub url: Url,
    pub host: String,
    pub addrs: Vec<SocketAddr>,
}

const ALLOWED_HOST_SUFFIXES: &[&str] = &["tenor.com", "giphy.com", "media.tenor.co"];

/// Parse, allowlist and resolve `url`. Rejects private/loopback/link-local/metadata IPs.
pub fn validate_public_https_url(url: &str) -> Result<Url, String> {
    let parsed = Url::parse(url).map_err(|e| format!("Invalid URL: {e}"))?;
    if parsed.scheme() != "https" {
        return Err("Only HTTPS URLs are allowed".into());
    }
    if parsed.username() != "" || parsed.password().is_some() {
        return Err("URLs with credentials are not allowed".into());
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| "URL is missing a host".to_string())?
        .to_ascii_lowercase();

    if host.parse::<std::net::IpAddr>().is_ok() || looks_like_dotted_ipv4(&host) {
        return Err("Direct IP downloads are not allowed".into());
    }
    if !is_allowed_host(&host) {
        return Err(format!("Host '{host}' is not on the download allowlist"));
    }

    let port = parsed.port().unwrap_or(443);
    let addrs = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|e| format!("DNS resolution failed for {host}: {e}"))?;

    let mut resolved = false;
    for addr in addrs {
        resolved = true;
        if is_disallowed_ip(addr.ip()) {
            return Err(format!(
                "Refusing download: {host} resolved to non-public IP {}",
                addr.ip()
            ));
        }
    }
    if !resolved {
        return Err(format!("No DNS records for {host}"));
    }

    Ok(parsed)
}

/// Validate `url` and return the parsed URL plus the public addresses it resolved to.
pub fn validate_and_pin(url: &str) -> Result<ValidatedDownload, String> {
    let parsed = validate_public_https_url(url)?;
    let host = parsed
        .host_str()
        .ok_or_else(|| "URL is missing a host".to_string())?
        .to_ascii_lowercase();
    let port = parsed.port().unwrap_or(443);
    let addrs: Vec<SocketAddr> = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|e| format!("DNS resolution failed for {host}: {e}"))?
        .collect();
    if addrs.is_empty() {
        return Err(format!("No DNS records for {host}"));
    }
    for addr in &addrs {
        if is_disallowed_ip(addr.ip()) {
            return Err(format!(
                "Refusing download: {host} resolved to non-public IP {}",
                addr.ip()
            ));
        }
    }
    Ok(ValidatedDownload {
        url: parsed,
        host,
        addrs,
    })
}

/// HTTP client that pins DNS to the already-validated public addresses.
/// Compiled only with `--features gif-search`.
/// کلاینت HTTP که DNS را به آدرس‌های تأییدشده پین می‌کند.
/// فقط با `--features gif-search` کامپایل می‌شود.
#[cfg(feature = "gif-search")]
pub fn pinned_client(validated: &ValidatedDownload, timeout: Duration) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(timeout)
        .https_only(true)
        .redirect(no_redirects())
        .resolve_to_addrs(&validated.host, &validated.addrs)
        .build()
        .map_err(|e| format!("HTTP client: {e}"))
}

#[cfg(feature = "gif-search")]
pub fn pinned_blocking_client(
    validated: &ValidatedDownload,
    timeout: Duration,
) -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(timeout)
        .https_only(true)
        .redirect(no_redirects())
        .resolve_to_addrs(&validated.host, &validated.addrs)
        .build()
        .map_err(|e| format!("HTTP client: {e}"))
}

fn is_allowed_host(host: &str) -> bool {
    ALLOWED_HOST_SUFFIXES.iter().any(|suffix| {
        host == *suffix
            || host.ends_with(&format!(".{suffix}"))
    })
}

/// reqwest redirect policy: never follow. Callers must re-validate any new URL.
/// Compiled only with `--features gif-search` (reqwest is optional).
/// سیاست redirect: هرگز دنبال نکن. فقط با `--features gif-search`.
#[cfg(feature = "gif-search")]
pub fn no_redirects() -> reqwest::redirect::Policy {
    reqwest::redirect::Policy::none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_https_and_ips() {
        assert!(validate_public_https_url("http://media.tenor.com/x.gif").is_err());
        assert!(validate_public_https_url("https://127.0.0.1/x.gif").is_err());
        assert!(validate_public_https_url("https://169.254.169.254/latest").is_err());
        assert!(validate_public_https_url("https://evil.example/x.gif").is_err());
        assert!(validate_public_https_url("https://localhost/x.gif").is_err());
    }

    #[test]
    fn allowlist_recognises_tenor() {
        assert!(is_allowed_host("media.tenor.com"));
        assert!(is_allowed_host("media1.tenor.com"));
        assert!(is_allowed_host("c.tenor.com"));
        assert!(!is_allowed_host("tenor.com.evil.test"));
        assert!(!is_allowed_host("example.com"));
    }

    #[test]
    fn private_ips_are_blocked() {
        use crate::net_policy::is_disallowed_ip;
        assert!(is_disallowed_ip("10.0.0.1".parse().unwrap()));
        assert!(is_disallowed_ip("192.168.1.1".parse().unwrap()));
        assert!(is_disallowed_ip("127.0.0.1".parse().unwrap()));
        assert!(is_disallowed_ip("169.254.169.254".parse().unwrap()));
        assert!(is_disallowed_ip("100.64.0.1".parse().unwrap()));
        assert!(is_disallowed_ip("::1".parse().unwrap()));
        assert!(is_disallowed_ip("fc00::1".parse().unwrap()));
        assert!(is_disallowed_ip("fe80::1".parse().unwrap()));
        assert!(!is_disallowed_ip("8.8.8.8".parse().unwrap()));
        assert!(!is_disallowed_ip("1.1.1.1".parse().unwrap()));
    }
}
