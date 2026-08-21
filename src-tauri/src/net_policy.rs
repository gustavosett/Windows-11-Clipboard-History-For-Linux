//! Shared IP / host policy for SSRF defence and Smart-Action URL opening.
//! / سیاست مشترک IP و میزبان برای دفاع SSRF و باز کردن URL عملیات هوشمند.
//!
//! `ssrf.rs` and `open_url.rs` must not drift: a range blocked in one
//! path has to be blocked in the other. This module is the single source
//! of truth.
//! `ssrf.rs` و `open_url.rs` نباید از هم فاصله بگیرند؛ این ماژول منبع واحد است.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// True when `ip` must never be contacted (loopback, private, CGNAT,
/// link-local, multicast, documentation, unspecified).
/// وقتی `ip` هرگز نباید تماس گرفته شود «درست» است.
pub fn is_disallowed_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_disallowed_v4(v4),
        IpAddr::V6(v6) => {
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_disallowed_v4(v4);
            }
            is_disallowed_v6(v6)
        }
    }
}

/// True when `host` looks like a dotted IPv4 even if `IpAddr` parsing failed
/// (rejects leading-zero / truncated forms the URL crate may leave as a name).
/// وقتی میزبان شبیه IPv4 نقطه‌ای است حتی اگر تجزیهٔ `IpAddr` شکست بخورد.
pub fn looks_like_dotted_ipv4(host: &str) -> bool {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u8>().is_ok())
}

pub fn is_disallowed_v4(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || ip.is_multicast()
        || o[0] == 0
        || (o[0] == 100 && o[1] & 0b1100_0000 == 0b0100_0000) // 100.64.0.0/10 CGNAT
        || (o[0] == 192 && o[1] == 0 && o[2] == 0) // 192.0.0.0/24 IETF
        || (o[0] == 192 && o[1] == 0 && o[2] == 2) // TEST-NET-1
        || (o[0] == 198 && (o[1] == 18 || o[1] == 19)) // benchmark
        || (o[0] == 198 && o[1] == 51 && o[2] == 100) // TEST-NET-2
        || (o[0] == 203 && o[1] == 0 && o[2] == 113) // TEST-NET-3
        || o[0] >= 224 // multicast + reserved + broadcast
}

pub fn is_disallowed_v6(ip: Ipv6Addr) -> bool {
    let s = ip.segments();
    ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || (s[0] & 0xffc0) == 0xfe80 // link-local fe80::/10
        || (s[0] & 0xfe00) == 0xfc00 // unique local fc00::/7
        || (s[0] == 0x2001 && s[1] == 0x0db8) // documentation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_and_special_ips_are_blocked() {
        assert!(is_disallowed_ip("10.0.0.1".parse().unwrap()));
        assert!(is_disallowed_ip("192.168.1.1".parse().unwrap()));
        assert!(is_disallowed_ip("127.0.0.1".parse().unwrap()));
        assert!(is_disallowed_ip("169.254.169.254".parse().unwrap()));
        assert!(is_disallowed_ip("100.64.0.1".parse().unwrap()));
        assert!(is_disallowed_ip("224.0.0.1".parse().unwrap()));
        assert!(is_disallowed_ip("0.0.0.0".parse().unwrap()));
        assert!(is_disallowed_ip("::1".parse().unwrap()));
        assert!(is_disallowed_ip("fc00::1".parse().unwrap()));
        assert!(is_disallowed_ip("fe80::1".parse().unwrap()));
        assert!(!is_disallowed_ip("8.8.8.8".parse().unwrap()));
        assert!(!is_disallowed_ip("1.1.1.1".parse().unwrap()));
    }

    #[test]
    fn dotted_ipv4_detection() {
        assert!(looks_like_dotted_ipv4("127.0.0.1"));
        assert!(looks_like_dotted_ipv4("8.8.8.8"));
        assert!(!looks_like_dotted_ipv4("example.com"));
        assert!(!looks_like_dotted_ipv4("1.2.3"));
    }
}
