use aoike_leptos::{AoikeApp, ConfigContext};
use leptos::prelude::*;

pub fn main() {
    console_error_panic_hook::set_once();

    let config = ConfigContext {
        title: Some("Aoike Leptos Example".to_string()),
        desc: Some("An example site built with Aoike and Leptos".to_string()),
        // Using a placeholder avatar
        avatar: Some("avatar.jpg".to_string()),
        github_owner: Some("aoike".to_string()),
        github_repo: Some("aoike".to_string()),
        // vault_base_url: Some("/vault".to_string()),
        vault_base_url: Some("http://127.0.0.1:8080/".to_string()),
        giscus_options: None,
        // giscus_options: Some(GiscusOptions::new(
        //     "your-repo".to_string(),
        //     "your-repo-id".to_string(),
        //     "your-category-id".to_string(),
        // )),
        ..Default::default()
    };

    mount_to_body(move || {
        view! { <AoikeApp config=config /> }
    })
}
