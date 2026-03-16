//! Fetch HTML content from a URL or read from a local file.

use crate::error::{BoundformError, Result};
use reqwest::blocking::Client;
use reqwest::header::{COOKIE, HeaderMap, HeaderName, HeaderValue};
use reqwest::redirect::Policy;
use std::net::ToSocketAddrs;
use std::path::Path;
use std::time::Duration;
use url::Url;

/// Maximum response body size (10 MB).
const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024;

/// Request timeout (30 seconds).
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Maximum number of redirects to follow.
const MAX_REDIRECTS: usize = 5;

/// HTTP client options for authentication and custom headers.
#[derive(Debug, Default, Clone)]
pub struct FetchOptions {
    /// Cookie values to attach (e.g., "authjs.session-token=eyJ...")
    pub cookies: Vec<String>,
    /// Custom headers to attach (e.g., "Authorization: Bearer eyJ...")
    pub headers: Vec<String>,
}

/// Fetch HTML from a URL or read from a local file path.
///
/// If `source` starts with `http://` or `https://`, it is treated as a URL.
/// Otherwise, it is treated as a local file path.
pub fn fetch_html(source: &str, options: &FetchOptions) -> Result<String> {
    if source.starts_with("http://") || source.starts_with("https://") {
        validate_url(source)?;
        fetch_from_url(source, options)
    } else {
        read_from_file(source)
    }
}

/// Validate that a URL is safe to fetch (no SSRF to internal services).
fn validate_url(url_str: &str) -> Result<()> {
    let parsed = Url::parse(url_str).map_err(|e| BoundformError::ConfigError(e.to_string()))?;

    // Only allow http and https schemes
    match parsed.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(BoundformError::ConfigError(format!(
                "unsupported URL scheme '{scheme}': only http and https are allowed"
            )));
        }
    }

    // Resolve the host and block private/loopback IPs
    if let Some(host) = parsed.host_str() {
        // Block obvious private hostnames
        let lower = host.to_lowercase();
        if lower == "metadata.google.internal"
            || lower.ends_with(".internal")
            || lower == "instance-data"
        {
            return Err(BoundformError::ConfigError(format!(
                "blocked request to internal hostname: {host}"
            )));
        }

        // Resolve DNS and check IP addresses
        let port = parsed
            .port()
            .unwrap_or(if parsed.scheme() == "https" { 443 } else { 80 });
        let addr_str = format!("{host}:{port}");
        if let Ok(addrs) = addr_str.to_socket_addrs() {
            for addr in addrs {
                let ip = addr.ip();
                if ip.is_loopback() && lower != "localhost" && lower != "127.0.0.1" {
                    return Err(BoundformError::ConfigError(format!(
                        "blocked request to loopback address: {ip}"
                    )));
                }
                // Check for link-local (169.254.x.x) - common cloud metadata endpoint
                if let std::net::IpAddr::V4(v4) = ip
                    && v4.octets()[0] == 169
                    && v4.octets()[1] == 254
                {
                    return Err(BoundformError::ConfigError(format!(
                        "blocked request to link-local address: {ip}"
                    )));
                }
            }
        }
    }

    Ok(())
}

fn fetch_from_url(url: &str, options: &FetchOptions) -> Result<String> {
    let mut headers = HeaderMap::new();

    // Add cookies
    if !options.cookies.is_empty() {
        let cookie_str = options.cookies.join("; ");
        match HeaderValue::from_str(&cookie_str) {
            Ok(val) => {
                headers.insert(COOKIE, val);
            }
            Err(e) => {
                eprintln!(
                    "Warning: cookie value contains invalid characters and was not sent: {e}"
                );
            }
        }
    }

    // Add custom headers
    for header in &options.headers {
        if let Some((name, value)) = header.split_once(':') {
            let name = name.trim();
            let value = value.trim();
            match (
                HeaderName::from_bytes(name.as_bytes()),
                HeaderValue::from_str(value),
            ) {
                (Ok(header_name), Ok(header_value)) => {
                    headers.insert(header_name, header_value);
                }
                _ => {
                    eprintln!(
                        "Warning: header '{name}' contains invalid characters and was not sent"
                    );
                }
            }
        }
    }

    let client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .redirect(Policy::limited(MAX_REDIRECTS))
        .build()?;

    let response = client.get(url).headers(headers).send()?;

    // Check content length before reading body
    if let Some(len) = response.content_length()
        && len as usize > MAX_RESPONSE_SIZE
    {
        return Err(BoundformError::ConfigError(format!(
            "response too large: {} bytes (max: {} bytes)",
            len, MAX_RESPONSE_SIZE
        )));
    }

    let body = response.text()?;

    // Double-check after reading (content-length may be absent)
    if body.len() > MAX_RESPONSE_SIZE {
        return Err(BoundformError::ConfigError(format!(
            "response too large: {} bytes (max: {} bytes)",
            body.len(),
            MAX_RESPONSE_SIZE
        )));
    }

    Ok(body)
}

/// Read HTML from a local file with path validation.
fn read_from_file(path_str: &str) -> Result<String> {
    let path = Path::new(path_str);

    // Canonicalize the path to resolve .. and symlinks
    let canonical = path.canonicalize().map_err(BoundformError::IoError)?;

    // Block absolute paths that look like system files
    let path_string = canonical.to_string_lossy();
    #[cfg(unix)]
    {
        let blocked_prefixes = ["/etc/", "/proc/", "/sys/", "/dev/"];
        for prefix in &blocked_prefixes {
            if path_string.starts_with(prefix) {
                return Err(BoundformError::ConfigError(format!(
                    "blocked access to system path: {}",
                    path_string
                )));
            }
        }
    }
    #[cfg(windows)]
    {
        let lower = path_string.to_lowercase();
        if lower.starts_with("c:\\windows\\") || lower.starts_with("c:\\program") {
            return Err(BoundformError::ConfigError(format!(
                "blocked access to system path: {}",
                path_string
            )));
        }
    }

    // Check file size before reading
    let metadata = std::fs::metadata(&canonical)?;
    if metadata.len() as usize > MAX_RESPONSE_SIZE {
        return Err(BoundformError::ConfigError(format!(
            "file too large: {} bytes (max: {} bytes)",
            metadata.len(),
            MAX_RESPONSE_SIZE
        )));
    }

    let content = std::fs::read_to_string(&canonical)?;
    Ok(content)
}
