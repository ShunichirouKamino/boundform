//! Fetch HTML content from a URL or read from a local file.

use crate::error::Result;
use reqwest::blocking::Client;
use reqwest::header::{COOKIE, HeaderMap, HeaderName, HeaderValue};
use std::path::Path;

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
        fetch_from_url(source, options)
    } else {
        read_from_file(source)
    }
}

fn fetch_from_url(url: &str, options: &FetchOptions) -> Result<String> {
    let mut headers = HeaderMap::new();

    // Add cookies
    if !options.cookies.is_empty() {
        let cookie_str = options.cookies.join("; ");
        if let Ok(val) = HeaderValue::from_str(&cookie_str) {
            headers.insert(COOKIE, val);
        }
    }

    // Add custom headers
    for header in &options.headers {
        if let Some((name, value)) = header.split_once(':') {
            let name = name.trim();
            let value = value.trim();
            if let (Ok(header_name), Ok(header_value)) = (
                HeaderName::from_bytes(name.as_bytes()),
                HeaderValue::from_str(value),
            ) {
                headers.insert(header_name, header_value);
            }
        }
    }

    let client = Client::new();
    let response = client.get(url).headers(headers).send()?;
    let body = response.text()?;
    Ok(body)
}

fn read_from_file(path: &str) -> Result<String> {
    let content = std::fs::read_to_string(Path::new(path))?;
    Ok(content)
}
