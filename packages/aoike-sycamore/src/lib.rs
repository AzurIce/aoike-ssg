#[cfg(feature = "build")]
pub mod build;

pub mod components {
    pub mod giscus;
}

pub mod layout {
    pub mod base;
}

use aoike::{Article, Id, Vault};
#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
use sycamore::prelude::*;
use sycamore::web::{Suspense, create_client_resource};
use sycamore_router::{HistoryIntegration, Route, Router, navigate};

use crate::{components::giscus::GiscusOptions, layout::base::Header};

#[derive(Route, Clone)]
enum AppRoutes {
    #[to("/")]
    Index,
    #[to("/posts")]
    Posts,
    #[to("/posts/<slug>")]
    Post { slug: String },
    #[not_found]
    NotFound,
}

#[derive(Clone, PartialEq, Eq, Default)]
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

#[cfg(target_arch = "wasm32")]
async fn fetch_vault(base_url: &str) -> Result<Vault, String> {
    let url = format!("{}/vault.json", base_url.trim_end_matches('/'));
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_vault(_base_url: &str) -> Result<Vault, String> {
    Err("Fetching vault is not supported on non-WASM targets".to_string())
}

#[cfg(target_arch = "wasm32")]
async fn fetch_article(base_url: &str, path: &str) -> Result<Article, String> {
    let url = format!("{}/articles/{}.json", base_url.trim_end_matches('/'), path);
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
async fn fetch_article(_base_url: &str, _path: &str) -> Result<Article, String> {
    Err("Fetching article is not supported on non-WASM targets".to_string())
}

#[component(inline_props)]
pub fn AoikeApp(config: ConfigContext) -> View {
    provide_context(config.clone());

    let base_url = config
        .vault_base_url
        .clone()
        .unwrap_or_else(|| "/vault".to_string());

    let vault_resource = create_client_resource(move || {
        let base_url = base_url.clone();
        async move { fetch_vault(&base_url).await }
    });

    view! {
        // Suspense(fallback=move || view! { "Loading..." }) {
            (if let Some(vault) = vault_resource.get_clone() {
                match vault {
                    Ok(vault) => {
                        provide_context(vault.clone());
                        view! {
                            Router(
                                integration=HistoryIntegration::new(),
                                view=move |route: ReadSignal<AppRoutes>| {
                                    view! {
                                        Header()

                                        main(class="max-w-[80ch] w-full m-x-auto flex flex-col items-center p-8 gap-4") {
                                            (match route.get_clone() {
                                                AppRoutes::Index => view! {
                                                    Index()
                                                },
                                                AppRoutes::Posts => view! {
                                                    Posts()
                                                },
                                                AppRoutes::Post { slug } => view! {
                                                    Post(slug=slug)
                                                },
                                                AppRoutes::NotFound => view! {
                                                    NotFoundPage()
                                                },
                                            })
                                        }
                                    }
                                }
                            )
                        }
                    }
                    Err(err) => {
                        let msg = format!("Error: {err:?}");
                        view! { (msg) }
                    }
                }
            } else {
                view! { "loading..." }
            })
        // }
    }
}

#[component]
pub fn Index() -> View {
    let config = use_context::<ConfigContext>();
    let vault = use_context::<Vault>();

    let base_url = config
        .vault_base_url
        .clone()
        .unwrap_or_else(|| "/vault".to_string());

    let index_article_resource = create_client_resource(move || {
        let base_url = base_url.clone();
        async move { fetch_article(&base_url, "posts/index").await }
    });

    let recent_posts_view = vault
        .posts
        .articles
        .iter()
        .take(5)
        .cloned()
        .map(|blog| {
            view! {
                li(class="flex gap-8") {
                    span(class="text-gray-600") {
                        (format!("{}-{}-{}",
                            blog.created.year(),
                            u8::from(blog.created.month()),
                            blog.created.day()
                        ))
                    }
                    a(
                        class="underline hover:underline-gray-400",
                        href=format!("/posts/{}", blog.id)
                    ) {
                        (blog.title.clone())
                    }
                }
            }
        })
        .collect::<Vec<View>>();

    view! {
        Hero()

        div(class="flex flex-col w-full p-2 markdown") {
            h2 { "最新文章" }
            ul {
                (recent_posts_view)
            }
            hr {}
            Suspense(fallback=move || view! { "Loading content..." }) {
                (index_article_resource.with(|val| {
                    match val {
                        Some(Ok(article)) => {
                            let html = article.content_html.clone();
                            view! {
                                div(dangerously_set_inner_html=html)
                            }
                        },
                        Some(Err(e)) => {
                            let error_msg = format!("Error loading content: {}", e);
                            view! { (error_msg) }
                        },
                        None => view! { }
                    }
                }))
            }
        }

        (config.giscus_options.clone().map(|options| {
            view! { components::giscus::Giscus(options=options) }
        }))
    }
}

#[component]
pub fn Hero() -> View {
    let config = use_context::<ConfigContext>();

    let title = config.title.as_deref().unwrap_or("Site Title").to_string();
    let desc = config
        .desc
        .as_deref()
        .unwrap_or("site description")
        .to_string();

    view! {
        div(class="flex items-stretch") {
            (config.avatar.clone().map(|avatar| {
                view! {
                    img(class="size-40 rounded", src=avatar)
                }
            }))

            div(class="flex flex-col items-center justify-around p-2 p-b-1 gap-3") {
                span(class="text-xl lxgw") {
                    "< " (title) " />"
                }

                span(class="text-sm lxgw") {
                    (desc)
                }

                (config.email.clone().map(|email| {
                    let _email = email.clone();
                    view! {
                        span(class="text-sm") {
                            "📫 "
                            a(class="underline", href=format!("mailto:{}", email)) {
                                (_email)
                            }
                        }
                    }
                }))

                div(class="flex") {
                    (config.github_owner.clone().map(|owner| {
                        view! {
                            a(href=format!("https://github.com/{}", owner), target="_blank", rel="noreferrer", class="size-8 gap-1 nav-btn") {
                                div(class="i-fa6-brands-github text-xl")
                            }
                        }
                    }))

                    (config.bilibili_url.clone().map(|url| {
                        view! {
                            a(href=url, target="_blank", rel="noreferrer", class="size-8 gap-1 nav-btn") {
                                div(class="i-fa6-brands-bilibili text-xl color-[#19a2d4] translate-x-0 translate-y-[1px]")
                            }
                        }
                    }))

                    (config.steam_url.clone().map(|url| {
                        view! {
                            a(href=url, target="_blank", rel="noreferrer", class="size-8 gap-1 nav-btn") {
                                div(class="i-fa6-brands-steam text-xl bg-[#082256]")
                            }
                        }
                    }))
                }
            }
        }
    }
}

#[component]
pub fn Posts() -> View {
    let vault = use_context::<Vault>();
    view! {
        h1 { "所有文章" }
        (vault.posts.articles.iter().cloned().map(|post| {
            view! {
                PostCard(post=post)
            }
        }).collect::<Vec<_>>())
    }
}

#[component(inline_props)]
pub fn PostCard(post: Article) -> View {
    let summary_html = post.summary_html.clone();
    view! {
        div(
            class="w-full flex flex-col gap-2 p-2 rounded border border-slate-200 hover:border-slate-400"
        ) {
            a(href=format!("/posts/{}", post.id.clone())) {
                h2 { (post.title.clone()) }
            }
            div(class="flex gap-2") {
                span(class="text-xs text-gray-400") {
                    "创建日期: " (format!("{}-{}-{}",
                        post.created.year(),
                        u8::from(post.created.month()),
                        post.created.day()
                    ))
                }
                span(class="text-xs text-gray-400") {
                    "更新日期: " (format!("{}-{}-{}",
                        post.updated.year(),
                        u8::from(post.updated.month()),
                        post.updated.day()
                    ))
                }
            }
            div(class="summary", dangerously_set_inner_html=summary_html)
        }
    }
}

#[component(inline_props)]
pub fn Post(slug: String) -> View {
    let id = Id::new(slug);
    let config = use_context::<ConfigContext>();
    let vault = use_context::<Vault>();

    // Verify slug exists in vault to avoid unnecessary fetch or show 404 earlier?
    // But fetches are cheapish, maybe just fetch.
    // Actually, we need to know the path to fetch.
    // For posts, it's articles/posts/{slug}.json

    let exists = vault.posts.articles.iter().any(|p| p.id == id);
    if !exists {
        navigate("/404");
        return view! {};
    }

    let base_url = config
        .vault_base_url
        .clone()
        .unwrap_or_else(|| "/vault".to_string());

    let fetch_id = id.clone();
    let article_resource = create_client_resource(move || {
        let base_url = base_url.clone();
        let id = fetch_id.clone();
        async move { fetch_article(&base_url, &format!("posts/{}", id)).await }
    });

    let giscus_options = config.giscus_options.clone();

    view! {
        Suspense(fallback=move || view! { "Loading article..." }) {
             (
                match article_resource.get_clone() {
                    Some(Ok(article)) => {
                        let html = article.content_html.clone();
                        let giscus_opts = giscus_options.clone();
                        view! {
                            div(class="markdown w-full") {
                                div(dangerously_set_inner_html=html)
                            }

                            (giscus_opts.clone().map(|options| {
                                view! { components::giscus::Giscus(options=options) }
                            }))
                        }
                    },
                    Some(Err(e)) => {
                        let error_msg = format!("Error loading article: {}", e);
                        view! { (error_msg) }
                    },
                    None => view! { }
                }
             )
        }
    }
}

#[component]
pub fn NotFoundPage() -> View {
    view! {
        h1 { "404 Not Found" }
        p { "The page you're looking for doesn't exist." }
    }
}
