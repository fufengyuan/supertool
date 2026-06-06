use std::fmt::Display;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, ToSocketAddrs};
use std::str::FromStr;
use std::sync::Mutex;

use async_trait::async_trait;
use indexmap::IndexMap;
use serde_json::{json, Value};
use url::Url;

use hermes_core::{tool_schema, JsonSchema, ToolError, ToolHandler, ToolSchema};

const METADATA_HOSTNAMES: [&str; 3] = [
    "metadata.google.internal",
    "metadata.goog",
    "metadata.internal",
];
const QQ_MULTIMEDIA_HOST: &str = "multimedia.nt.qq.com.cn";

static ALLOW_PRIVATE_URLS_CACHE: Mutex<Option<bool>> = Mutex::new(None);

pub struct UrlSafetyHandler;

fn parse_bool_like(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
fn config_allow_private_urls() -> bool {
    false
}

#[cfg(not(test))]
fn config_allow_private_urls() -> bool {
    match hermes_config::load_config(None) {
        Ok(cfg) => {
            if cfg.security.allow_private_urls {
                return true;
            }
            cfg.tools_config
                .per_tool
                .get("browser")
                .and_then(|v| v.get("allow_private_urls"))
                .and_then(Value::as_bool)
                .unwrap_or(false)
        }
        Err(_) => false,
    }
}

fn global_allow_private_urls() -> bool {
    let mut guard = ALLOW_PRIVATE_URLS_CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(v) = *guard {
        return v;
    }

    let resolved = std::env::var("HERMES_ALLOW_PRIVATE_URLS")
        .ok()
        .and_then(|v| parse_bool_like(&v))
        .unwrap_or_else(config_allow_private_urls);
    *guard = Some(resolved);
    resolved
}

#[cfg(test)]
fn reset_allow_private_cache_for_tests() {
    let mut guard = ALLOW_PRIVATE_URLS_CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *guard = None;
}

fn is_benchmark_ipv4(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 198 && (18..=19).contains(&octets[1])
}

fn is_blocked_ipv4(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 10
        || (octets[0] == 172 && (16..=31).contains(&octets[1]))
        || (octets[0] == 192 && octets[1] == 168)
        || octets[0] == 127
        || (octets[0] == 169 && octets[1] == 254)
        || octets[0] == 0
        || (octets[0] == 100 && (64..=127).contains(&octets[1]))
        || is_benchmark_ipv4(ip)
        || ip.is_multicast()
        || ip.is_broadcast()
}

fn mapped_ipv4(ip: &Ipv6Addr) -> Option<Ipv4Addr> {
    ip.to_ipv4_mapped()
}

fn is_blocked_ipv6(ip: &Ipv6Addr) -> bool {
    if let Some(v4) = mapped_ipv4(ip) {
        return is_blocked_ipv4(&v4);
    }

    let segments = ip.segments();
    ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || (segments[0] & 0xffc0) == 0xfe80
        || (segments[0] & 0xfe00) == 0xfc00
}

fn is_blocked_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_ipv4(v4),
        IpAddr::V6(v6) => is_blocked_ipv6(v6),
    }
}

fn is_always_blocked_ipv4(ip: &Ipv4Addr) -> bool {
    let octets = ip.octets();
    (octets[0] == 169 && octets[1] == 254) || octets == [100, 100, 100, 200]
}

fn is_always_blocked_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_always_blocked_ipv4(v4),
        IpAddr::V6(v6) => {
            if let Some(v4) = mapped_ipv4(v6) {
                return is_always_blocked_ipv4(&v4);
            }
            *v6 == Ipv6Addr::new(0xfd00, 0x0ec2, 0, 0, 0, 0, 0, 0x0254)
        }
    }
}

fn is_metadata_hostname(host: &str) -> bool {
    METADATA_HOSTNAMES.contains(&host)
}

fn qq_multimedia_benchmark_exception(host: &str, scheme: &str, ip: &IpAddr) -> bool {
    if host != QQ_MULTIMEDIA_HOST || scheme != "https" {
        return false;
    }
    match ip {
        IpAddr::V4(v4) => is_benchmark_ipv4(v4),
        IpAddr::V6(v6) => mapped_ipv4(v6).as_ref().is_some_and(is_benchmark_ipv4),
    }
}

fn resolve_host_ips(host: &str, port: u16) -> Result<Vec<IpAddr>, std::io::Error> {
    let ips: Vec<IpAddr> = (host, port)
        .to_socket_addrs()?
        .map(|addr| addr.ip())
        .collect();
    if ips.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "host resolved to no addresses",
        ));
    }
    Ok(ips)
}

fn ip_allowed(host: &str, scheme: &str, ip: &IpAddr, allow_private_urls: bool) -> bool {
    if is_always_blocked_ip(ip) {
        return false;
    }
    allow_private_urls || !is_blocked_ip(ip) || qq_multimedia_benchmark_exception(host, scheme, ip)
}

fn is_safe_url_with_resolver<F, E>(url: &str, mut resolver: F) -> bool
where
    F: FnMut(&str, u16) -> Result<Vec<IpAddr>, E>,
    E: Display,
{
    let parsed = match Url::parse(url) {
        Ok(parsed) => parsed,
        Err(_) => return false,
    };
    let scheme = parsed.scheme();
    if !matches!(scheme, "http" | "https") {
        return false;
    }

    let Some(host) = parsed.host_str() else {
        return false;
    };
    let host = host.to_ascii_lowercase();
    if is_metadata_hostname(&host) {
        return false;
    }

    let allow_private_urls = global_allow_private_urls();
    if let Ok(ip) = IpAddr::from_str(&host) {
        return ip_allowed(&host, scheme, &ip, allow_private_urls);
    }

    let port = parsed.port_or_known_default().unwrap_or(80);
    let ips = match resolver(&host, port) {
        Ok(ips) if !ips.is_empty() => ips,
        Ok(_) | Err(_) => return false,
    };
    ips.iter()
        .all(|ip| ip_allowed(&host, scheme, ip, allow_private_urls))
}

pub fn is_safe_url(url: &str) -> bool {
    is_safe_url_with_resolver(url, resolve_host_ips)
}

fn is_always_blocked_url_with_resolver<F, E>(url: &str, mut resolver: F) -> bool
where
    F: FnMut(&str, u16) -> Result<Vec<IpAddr>, E>,
    E: Display,
{
    let parsed = match Url::parse(url) {
        Ok(parsed) => parsed,
        Err(_) => return false,
    };
    let Some(host) = parsed.host_str() else {
        return false;
    };
    let host = host.to_ascii_lowercase();
    if is_metadata_hostname(&host) {
        return true;
    }
    if let Ok(ip) = IpAddr::from_str(&host) {
        return is_always_blocked_ip(&ip);
    }

    let port = parsed.port_or_known_default().unwrap_or(80);
    resolver(&host, port)
        .map(|ips| ips.iter().any(is_always_blocked_ip))
        .unwrap_or(false)
}

pub fn is_always_blocked_url(url: &str) -> bool {
    is_always_blocked_url_with_resolver(url, resolve_host_ips)
}

#[async_trait]
impl ToolHandler for UrlSafetyHandler {
    async fn execute(&self, params: Value) -> Result<String, ToolError> {
        let url = params.get("url").and_then(|v| v.as_str()).unwrap_or("");
        if url.is_empty() {
            return Err(ToolError::InvalidParams("Missing 'url'".into()));
        }

        let safe = is_safe_url(url);
        let reason = if safe {
            "ok"
        } else if is_always_blocked_url(url) {
            "always_blocked"
        } else {
            "blocked_or_invalid"
        };
        Ok(json!({"url":url,"safe":safe,"reason":reason}).to_string())
    }

    fn schema(&self) -> ToolSchema {
        let mut props = IndexMap::new();
        props.insert("url".into(), json!({"type":"string"}));
        tool_schema(
            "url_safety",
            "Check whether a URL is safe to access.",
            JsonSchema::object(props, vec!["url".into()]),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_TEST_LOCK: Mutex<()> = Mutex::new(());

    struct EnvVarGuard {
        key: &'static str,
        old: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let old = std::env::var(key).ok();
            std::env::set_var(key, value);
            reset_allow_private_cache_for_tests();
            Self { key, old }
        }

        fn remove(key: &'static str) -> Self {
            let old = std::env::var(key).ok();
            std::env::remove_var(key);
            reset_allow_private_cache_for_tests();
            Self { key, old }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.old {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
            reset_allow_private_cache_for_tests();
        }
    }

    fn ips(values: &[&str]) -> Vec<IpAddr> {
        values
            .iter()
            .map(|value| value.parse().expect("valid ip"))
            .collect()
    }

    fn with_ips(
        values: &'static [&'static str],
    ) -> impl FnMut(&str, u16) -> Result<Vec<IpAddr>, String> {
        move |_, _| Ok(ips(values))
    }

    fn with_ip(value: &'static str) -> impl FnMut(&str, u16) -> Result<Vec<IpAddr>, String> {
        move |_, _| Ok(ips(&[value]))
    }

    fn resolver_err(_: &str, _: u16) -> Result<Vec<IpAddr>, String> {
        Err("dns failure".to_string())
    }

    #[test]
    fn safe_url_allows_public_http_and_https_only() {
        let _lock = ENV_TEST_LOCK.lock().expect("lock env");
        let _env = EnvVarGuard::remove("HERMES_ALLOW_PRIVATE_URLS");
        assert!(is_safe_url_with_resolver(
            "https://example.com/image.png",
            with_ips(&["93.184.216.34"])
        ));
        assert!(is_safe_url_with_resolver(
            "http://example.com/image.png",
            with_ips(&["93.184.216.34"])
        ));
        assert!(!is_safe_url_with_resolver(
            "ftp://example.com/file.txt",
            with_ips(&["93.184.216.34"])
        ));
        assert!(!is_safe_url_with_resolver(
            "example.com/path",
            with_ips(&["93.184.216.34"])
        ));
        assert!(!is_safe_url_with_resolver("http://", with_ips(&[])));
        assert!(!is_safe_url_with_resolver("", with_ips(&[])));
    }

    #[test]
    fn safe_url_blocks_private_reserved_and_metadata_ips() {
        let _lock = ENV_TEST_LOCK.lock().expect("lock env");
        let _env = EnvVarGuard::remove("HERMES_ALLOW_PRIVATE_URLS");
        for ip in [
            "127.0.0.1",
            "10.0.0.1",
            "172.16.0.1",
            "192.168.1.1",
            "169.254.169.254",
            "0.0.0.0",
            "100.64.0.1",
            "100.127.255.254",
            "198.18.0.23",
            "224.0.0.251",
            "255.255.255.255",
            "::1",
            "fe80::1",
            "fd12::1",
            "ff02::1",
            "::ffff:127.0.0.1",
            "::ffff:169.254.169.254",
            "::ffff:100.100.100.200",
        ] {
            let parsed: IpAddr = ip.parse().expect("valid ip");
            assert!(is_blocked_ip(&parsed), "{ip} should be blocked");
            assert!(!is_safe_url_with_resolver(
                "https://resolved.example/file",
                with_ip(ip)
            ));
        }
    }

    #[test]
    fn safe_url_allows_public_and_non_cgnat_100_addresses() {
        let _lock = ENV_TEST_LOCK.lock().expect("lock env");
        let _env = EnvVarGuard::remove("HERMES_ALLOW_PRIVATE_URLS");
        for ip in [
            "8.8.8.8",
            "93.184.216.34",
            "1.1.1.1",
            "100.0.0.1",
            "2606:4700::1",
            "2001:4860:4860::8888",
            "::ffff:8.8.8.8",
        ] {
            let parsed: IpAddr = ip.parse().expect("valid ip");
            assert!(!is_blocked_ip(&parsed), "{ip} should be allowed");
            assert!(is_safe_url_with_resolver(
                "https://resolved.example/file",
                with_ip(ip)
            ));
        }
    }

    #[test]
    fn qq_multimedia_exact_https_exception_allows_benchmark_ip() {
        let _lock = ENV_TEST_LOCK.lock().expect("lock env");
        let _env = EnvVarGuard::remove("HERMES_ALLOW_PRIVATE_URLS");
        assert!(is_safe_url_with_resolver(
            "https://multimedia.nt.qq.com.cn/download?id=123",
            with_ips(&["198.18.0.23"])
        ));
        assert!(!is_safe_url_with_resolver(
            "https://sub.multimedia.nt.qq.com.cn/download?id=123",
            with_ips(&["198.18.0.23"])
        ));
        assert!(!is_safe_url_with_resolver(
            "http://multimedia.nt.qq.com.cn/download?id=123",
            with_ips(&["198.18.0.23"])
        ));
        assert!(!is_safe_url_with_resolver(
            "https://multimedia.nt.qq.com.cn/download?id=123",
            resolver_err
        ));
    }

    #[test]
    fn safe_url_blocks_metadata_hostnames_without_dns() {
        let _lock = ENV_TEST_LOCK.lock().expect("lock env");
        let _env = EnvVarGuard::set("HERMES_ALLOW_PRIVATE_URLS", "true");
        assert!(!is_safe_url_with_resolver(
            "http://metadata.google.internal/computeMetadata/v1/",
            resolver_err
        ));
        assert!(!is_safe_url_with_resolver(
            "http://metadata.goog/computeMetadata/v1/",
            resolver_err
        ));
    }

    #[test]
    fn allow_private_toggle_permits_private_but_not_metadata() {
        let _lock = ENV_TEST_LOCK.lock().expect("lock env");
        let _env = EnvVarGuard::set("HERMES_ALLOW_PRIVATE_URLS", "true");
        assert!(is_safe_url_with_resolver(
            "http://router.local",
            with_ips(&["192.168.1.1"])
        ));
        assert!(is_safe_url_with_resolver(
            "http://tailscale-peer.example",
            with_ips(&["100.100.100.100"])
        ));
        assert!(is_safe_url_with_resolver(
            "http://localhost:8080/api",
            with_ips(&["127.0.0.1"])
        ));
        assert!(!is_safe_url_with_resolver(
            "http://169.254.169.254/latest/meta-data/",
            with_ips(&["169.254.169.254"])
        ));
        assert!(!is_safe_url_with_resolver(
            "http://[fd00:ec2::254]/latest/",
            with_ips(&["fd00:ec2::254"])
        ));
        assert!(!is_safe_url_with_resolver(
            "https://nonexistent.example.com",
            resolver_err
        ));
    }

    #[test]
    fn env_toggle_false_values_keep_private_blocked_and_cache_result() {
        let _lock = ENV_TEST_LOCK.lock().expect("lock env");
        let _env = EnvVarGuard::set("HERMES_ALLOW_PRIVATE_URLS", "false");
        assert!(!is_safe_url_with_resolver(
            "http://router.local",
            with_ips(&["192.168.1.1"])
        ));
        std::env::set_var("HERMES_ALLOW_PRIVATE_URLS", "true");
        assert!(!is_safe_url_with_resolver(
            "http://router.local",
            with_ips(&["192.168.1.1"])
        ));
    }

    #[test]
    fn always_blocked_url_floor_is_metadata_only() {
        assert!(is_always_blocked_url_with_resolver(
            "http://169.254.169.254/latest/meta-data/",
            with_ips(&["8.8.8.8"])
        ));
        assert!(is_always_blocked_url_with_resolver(
            "http://metadata.google.internal/",
            resolver_err
        ));
        assert!(is_always_blocked_url_with_resolver(
            "http://attacker.example/",
            with_ips(&["169.254.42.1"])
        ));
        assert!(!is_always_blocked_url_with_resolver(
            "http://127.0.0.1:8080/",
            with_ips(&["127.0.0.1"])
        ));
        assert!(!is_always_blocked_url_with_resolver(
            "http://100.64.0.1/",
            with_ips(&["100.64.0.1"])
        ));
        assert!(!is_always_blocked_url_with_resolver(
            "http://nonexistent.example.com/",
            resolver_err
        ));
        assert!(!is_always_blocked_url_with_resolver("", resolver_err));
    }

    #[tokio::test]
    async fn url_safety_handler_reports_safe_and_blocked_urls() {
        let handler = UrlSafetyHandler;
        let blocked = handler
            .execute(json!({"url":"http://169.254.169.254/latest/meta-data/"}))
            .await
            .expect("handler result");
        let blocked: Value = serde_json::from_str(&blocked).expect("json");
        assert_eq!(blocked["safe"], false);
        assert_eq!(blocked["reason"], "always_blocked");

        let missing = handler.execute(json!({})).await;
        assert!(matches!(missing, Err(ToolError::InvalidParams(_))));
    }
}
