use aoike::data::{ArticleDetail, VaultData};
use gloo_net::http::Request;
use leptos::leptos_dom::logging::console_log;

use crate::BASE_URL;

pub async fn fetch_vault(base_url: &str) -> Result<VaultData, gloo_net::Error> {
    let url = format!(
        "{BASE_URL}/static/{}/vault.json",
        base_url.trim_end_matches('/').trim_start_matches("/")
    );
    console_log(&format!("fetching from {url}"));
    let res = Request::get(&url).send().await?;
    res.json().await
}

pub async fn fetch_article(
    base_url: &str,
    ids_path: &str,
) -> Result<ArticleDetail, gloo_net::Error> {
    let url = format!(
        "{BASE_URL}/static/{}/articles/{ids_path}.json",
        base_url.trim_end_matches('/').trim_start_matches("/")
    );
    console_log(&format!("fetching from {url}"));
    let res = Request::get(&url).send().await?;
    res.json().await
}
