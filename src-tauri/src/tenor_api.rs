//! Tenor GIF API client (backend proxy).
//! The API key is read from TENOR_API_KEY. It is never bundled into the frontend.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

const TENOR_API_BASE: &str = "https://g.tenor.com/v1";
const TENOR_API_KEY_ENV: &str = "TENOR_API_KEY";

/// GIF data returned to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GifResult {
    pub id: String,
    pub title: String,
    pub preview_url: String,
    pub full_url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Deserialize)]
struct TenorV1Response {
    results: Vec<TenorV1Result>,
}

#[derive(Deserialize)]
struct TenorV1Result {
    id: String,
    title: Option<String>,
    content_description: Option<String>,
    media: Vec<TenorV1Media>,
}

#[derive(Deserialize)]
struct TenorV1Media {
    nanogif: Option<TenorV1Format>,
    tinygif: Option<TenorV1Format>,
    mediumgif: Option<TenorV1Format>,
    gif: Option<TenorV1Format>,
}

#[derive(Deserialize)]
struct TenorV1Format {
    url: String,
    dims: [u32; 2],
}

fn api_key() -> Result<String, String> {
    let key = std::env::var(TENOR_API_KEY_ENV).map_err(|_| {
        "TENOR_API_KEY is not set. GIF search is disabled until you export a Tenor API key.".to_string()
    })?;
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err("TENOR_API_KEY is empty".into());
    }
    Ok(key)
}

fn build_tenor_url(path: &str, key: &str, query: Option<&str>, limit: u32) -> Result<Url, String> {
    let mut url = Url::parse(&format!("{TENOR_API_BASE}/{path}"))
        .map_err(|e| format!("Invalid Tenor endpoint: {e}"))?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("key", key);
        pairs.append_pair("limit", &limit.to_string());
        pairs.append_pair("media_filter", "minimal");
        if let Some(q) = query {
            pairs.append_pair("q", q);
        }
    }
    Ok(url)
}

/// Search GIFs via Tenor API (server-side proxy).
/// Registered through `commands::search_tenor` so the default build can stub it.
/// جستجوی GIF از Tenor. ثبت از طریق `commands::search_tenor`.
pub async fn search_tenor(
    query: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<GifResult>, String> {
    let key = api_key()?;
    let limit = limit.unwrap_or(30).min(50);

    let url = if let Some(q) = query.as_ref().filter(|q| !q.trim().is_empty()) {
        build_tenor_url("search", &key, Some(q.trim()), limit)?
    } else {
        build_tenor_url("trending", &key, None, limit)?
    };

    let validated = crate::ssrf::validate_and_pin(url.as_str())?;
    let client = crate::ssrf::pinned_client(&validated, Duration::from_secs(10))?;

    let resp = client
        .get(validated.url)
        .send()
        .await
        .map_err(|e| format!("Network: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Tenor API HTTP {}", resp.status()));
    }

    let data: TenorV1Response = resp
        .json()
        .await
        .map_err(|e| format!("Parse error: {e}"))?;

    Ok(data
        .results
        .into_iter()
        .filter_map(|r| {
            let preview = r
                .media
                .first()?
                .nanogif
                .as_ref()
                .or(r.media.first()?.tinygif.as_ref())?;
            let full = r
                .media
                .first()?
                .tinygif
                .as_ref()
                .or(r.media.first()?.mediumgif.as_ref())
                .or(r.media.first()?.gif.as_ref())?;

            Some(GifResult {
                id: r.id,
                title: r
                    .content_description
                    .unwrap_or_else(|| r.title.unwrap_or_default()),
                preview_url: preview.url.clone(),
                full_url: full.url.clone(),
                width: preview.dims[0],
                height: preview.dims[1],
            })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_is_url_encoded() {
        let url = build_tenor_url("search", "abc", Some("cat & dog&key=stolen"), 10).unwrap();
        let query = url.query().unwrap_or("");
        assert!(query.contains("q=cat+%26+dog"));
        assert!(!query.contains("&key=stolen"));
        assert!(query.starts_with("key=abc"));
    }
}
