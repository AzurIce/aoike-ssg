pub mod api;
pub mod components {
    pub mod giscus;
}

pub mod layout {
    pub mod base;
}

pub mod routes {
    pub mod index;
    pub use index::Index;

    pub mod post;
    pub use post::{Post, Posts};

    // pub mod note;
    // pub use note::{Note, Notes};
}

use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use api::fetch_vault;
use components::giscus::GiscusOptions;
use layout::base::Header;

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

#[component]
pub fn AoikeApp(config: ConfigContext) -> impl IntoView {
    provide_context(config.clone());

    let base_url = config
        .vault_base_url
        .clone()
        .unwrap_or_else(|| "/vault".to_string());

    let vault_resource = LocalResource::new(move || {
        let base_url = base_url.clone();
        async move { fetch_vault(&base_url).await.ok() }
    });

    view! {
        <Router>
            <Header />
            <main class="max-w-[100ch] w-full m-x-auto flex flex-col items-center p-8 gap-4">
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
                                            <Routes fallback=|| view! { <NotFoundPage /> }>
                                                <Route path=path!("/") view=routes::Index />
                                                <Route path=path!("/posts") view=routes::Posts />
                                                <Route path=path!("/posts/:slug") view=routes::Post />
                                                // <Route path=path!("/notes") view=routes::Notes />
                                                // <Route path=path!("/notes/*path") view=routes::Note />
                                                <Route path=path!("/4o4") view=NotFoundPage />
                                            </Routes>
                                        }
                                            .into_any()
                                    }
                                    None => view! { <p>"Failed to load vault.json"</p> }.into_any(),
                                }
                            })
                    }}
                </Suspense>
            </main>
        </Router>
    }
}

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <h1>"404 Not Found"</h1>
        <p>"The page you're looking for doesn't exist."</p>
    }
}
