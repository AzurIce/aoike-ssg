use aoike::data::{ArticleData, VaultMeta};
use gloo_net::http::Request;
use leptos::leptos_dom::logging::console_log;

use crate::utils::based_url;

pub async fn fetch_vault(base_url: &str) -> Result<VaultMeta, gloo_net::Error> {
    let url = based_url(format!(
        "static/{}/vault.json",
        base_url.trim_end_matches('/').trim_start_matches("/")
    ));
    console_log(&format!("fetching from {url}"));
    let res = Request::get(&url).send().await?;
    res.json().await
}

pub async fn fetch_article(base_url: &str, ids_path: &str) -> Result<ArticleData, gloo_net::Error> {
    let url = based_url(format!(
        "static/{}/articles/{ids_path}.json",
        base_url.trim_end_matches('/').trim_start_matches("/")
    ));
    console_log(&format!("fetching from {url}"));
    let res = Request::get(&url).send().await?;
    res.json().await
}
