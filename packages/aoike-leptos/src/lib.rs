#![feature(const_option_ops)]
#![feature(const_trait_impl)]
pub mod api;
pub mod components {
    pub mod article;
    pub mod giscus;
}

pub mod layout {
    pub mod base;
    pub mod tri_column;
}

pub mod routes {
    pub mod index;
    pub use index::Index;

    pub mod post;
    pub use post::{Post, Posts};

    pub mod note;
    pub use note::{Note, Notes};
}

mod utils;
use leptos::{leptos_dom::logging::console_debug_log, prelude::*};
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use api::fetch_vault;
use components::giscus::GiscusOptions;
use layout::base::Header;

use crate::utils::mount_style;

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct ConfigContext {
    pub title: Option<String>,
    pub desc: Option<String>,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub github_owner: Option<String>,
    pub github_repo: Option<String>,
    pub bilibili_url: Option<String>,
    pub steam_url: Option<String>,
    pub giscus_options: Option<GiscusOptions>,
    pub vault_base_url: Option<String>,
}

const CSS_MAIN: &str = include_str!("../css/main.css");
const CSS_UNO: &str = include_str!("../css/uno.css");
// Should starts with "/" (absolute) and NOT end with "/"
const BASE_URL: &str = option_env!("TRUNK_BUILD_PUBLIC_URL").unwrap_or("");

#[component]
pub fn CssProvider(children: Children) -> impl IntoView {
    mount_style("aoike-main", CSS_MAIN);
    mount_style("aoike-uno", CSS_UNO);
    children()
}

#[component]
pub fn AoikeApp(config: ConfigContext) -> impl IntoView {
    provide_context(config.clone());
    console_debug_log(&format!("base url: {BASE_URL}"));

    let vault_resource = LocalResource::new(move || {
        let vault_base_url = config
            .vault_base_url
            .clone()
            .unwrap_or_else(|| "/vault".to_string());
        console_debug_log(&format!("Fetching vault from {vault_base_url}"));
        async move { fetch_vault(&vault_base_url).await.ok() }
    });

    provide_meta_context();

    view! {
        <CssProvider>
            <Suspense fallback=move || {
                view! { "Loading..." }
            }>
                {move || {
                    vault_resource
                        .get()
                        .map(|vault_res| {
                            match vault_res {
                                Some(vault) => {
                                    provide_context(vault.clone());
                                    view! {
                                        <Router base=BASE_URL>
                                            <Header />
                                            <Routes fallback=|| view! { <NotFoundPage /> }>
                                                <Route path=path!("/") view=routes::Index />
                                                <Route path=path!("/posts") view=routes::Posts />
                                                <Route path=path!("/posts/:slug") view=routes::Post />
                                                <Route path=path!("/notes") view=routes::Notes />
                                                <Route path=path!("/notes/*path") view=routes::Note />
                                                <Route path=path!("/4o4") view=NotFoundPage />
                                            </Routes>
                                        </Router>
                                    }
                                        .into_any()
                                }
                                None => view! { <p>"Failed to load vault.json"</p> }.into_any(),
                            }
                        })
                }}
            </Suspense>
        </CssProvider>
    }
}

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <h1>"404 Not Found"</h1>
        <p>"The page you're looking for doesn't exist."</p>
    }
}
