use aoike_sycamore::app::{
    AoikeApp, ConfigContext,
    components::giscus::{GiscusOptions, InputPosition},
};

use sycamore::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    sycamore::render(|| {
        view! {
            AoikeApp(
                config=ConfigContext {
                    title: Some("冰弦のBlog".to_string()),
                    desc: Some("『看清世界的真相后仍热爱生活』".to_string()),
                    email: Some("973562770@qq.com".to_string()),
                    avatar: Some("/posts/assets/avatar.jpg".to_string()), // Updated path to match probable asset location?
                    // actually if I use the old copy-file logic, it was at posts/assets/avatar.jpg

                    github_owner: Some("AzurIce".to_string()),
                    github_repo: Some("azurice.github.io".to_string()),
                    bilibili_url: Some("https://space.bilibili.com/46452693".to_string()),
                    steam_url: Some("https://steamcommunity.com/id/AzurIce".to_string()),
                    giscus_options: Some(
                        GiscusOptions::new(
                            "AzurIce/azurice.github.io".to_string(),
                            "R_kgDOI7WMeQ".to_string(),
                            "DIC_kwDOI7WMec4CUE3s".to_string(),
                        )
                        .with_category("Giscus".to_string())
                        .with_reactions_enabled(true)
                        .with_lazy(true)
                        .with_input_position(InputPosition::Top),
                    ),
                    vault_base_url: Some("/static/vault".to_string()),
                    ..Default::default()
                },
            )
        }
    });
}
