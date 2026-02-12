use aoike::data::{ArticleData, VaultMeta};
use gloo_net::http::Request;
use leptos::leptos_dom::logging::console_log;

use crate::utils::based_url;

/// Build a fetch URL for the vault.
/// If `base_url` is an absolute URL (http/https), use it directly.
/// Otherwise treat it as a relative path under `static/`.
fn vault_url(base_url: &str, path: &str) -> String {
    let base = base_url.trim_end_matches('/');
    if base.starts_with("http://") || base.starts_with("https://") {
        format!("{base}/{path}")
    } else {
        let base = base.trim_start_matches('/');
        based_url(format!("static/{base}/{path}"))
    }
}

pub async fn fetch_vault(base_url: &str) -> Result<VaultMeta, gloo_net::Error> {
    let url = vault_url(base_url, "vault.json");
    console_log(&format!("fetching from {url}"));
    let res = Request::get(&url).send().await?;
    res.json().await
}

pub async fn fetch_article(base_url: &str, ids_path: &str) -> Result<ArticleData, gloo_net::Error> {
    let url = vault_url(base_url, &format!("articles/{ids_path}.json"));
    console_log(&format!("fetching from {url}"));
    let res = Request::get(&url).send().await?;
    res.json().await
}
