use aoike::data::{ArticleDetail, VaultData};
use gloo_net::http::Request;

pub async fn fetch_vault(base_url: &str) -> Result<VaultData, gloo_net::Error> {
    let url = format!("{}/vault.json", base_url.trim_end_matches('/'));
    let res = Request::get(&url).send().await?;
    res.json().await
}

pub async fn fetch_article(base_url: &str, path: &str) -> Result<ArticleDetail, gloo_net::Error> {
    let url = format!("{}/articles/{}.json", base_url.trim_end_matches('/'), path);
    let res = Request::get(&url).send().await?;
    res.json().await
}
