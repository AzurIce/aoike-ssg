pub mod api;
pub mod components {
    pub mod giscus;
}

pub mod layout {
    pub mod base;
}

use aoike::data::{NodeMeta, PostMeta, VaultData};
use sycamore::prelude::*;
use sycamore::web::Suspense;
use sycamore_router::{HistoryIntegration, Route, Router, navigate};
use time::OffsetDateTime;

use api::{fetch_article, fetch_vault};
use components::giscus::GiscusOptions;
use layout::base::Header;

#[derive(Route, Clone)]
enum AppRoutes {
    #[to("/")]
    Index,
    #[to("/posts")]
    Posts,
    #[to("/posts/<slug>")]
    Post { slug: String },
    #[to("/notes")]
    Notes,
    #[to("/notes/<path..>")]
    Note { path: Vec<String> },
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

#[component(inline_props)]
pub async fn AoikeApp(config: ConfigContext) -> View {
    provide_context(config.clone());

    let base_url = config
        .vault_base_url
        .clone()
        .unwrap_or_else(|| "/vault".to_string());

    let vault = fetch_vault(&base_url).await;

    view! {
        Suspense(fallback=move || view! { "Loading..." }) {
            (
                match &vault {
                    Ok(vault) => {
                        provide_context(vault.clone());
                        view! {
                            Router(
                                integration=HistoryIntegration::new(),
                                view=move |route: ReadSignal<AppRoutes>| {
                                    view! {
                                        Header()

                                        main(class="max-w-[100ch] w-full m-x-auto flex flex-col items-center p-8 gap-4") {
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
                                                AppRoutes::Notes => view! {
                                                    Notes()
                                                },
                                                AppRoutes::Note { path } => view! {
                                                    Note(path=path)
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
            )
        }
    }
}

#[component]
pub async fn Index() -> View {
    let config = use_context::<ConfigContext>();
    let vault = use_context::<VaultData>();

    let base_url = config
        .vault_base_url
        .clone()
        .unwrap_or_else(|| "/vault".to_string());

    let index_article = fetch_article(&base_url, "posts").await;

    let recent_posts_view = vault
        .posts
        .iter()
        .take(5)
        .cloned()
        .map(|blog| {
            let created = OffsetDateTime::from_unix_timestamp(blog.created).unwrap();
            view! {
                li(class="flex gap-8") {
                    span(class="text-gray-600") {
                        (format!("{}-{}-{}",
                            created.year(),
                            u8::from(created.month()),
                            created.day()
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

    let content_view = match index_article {
        Ok(article) => {
            let html = article.content.clone();
            view! {
                div(dangerously_set_inner_html=html)
            }
        }
        Err(e) => {
            let error_msg = format!("Error loading content: {}", e);
            view! { (error_msg) }
        }
    };

    view! {
        Hero()

        div(class="flex flex-col w-full p-2 markdown") {
            h2 { "最新文章" }
            ul {
                (recent_posts_view)
            }
            hr {}
            Suspense(fallback=move || view! { "Loading content..." }) {
                (content_view)
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
    let vault = use_context::<VaultData>();
    view! {
        h1 { "所有文章" }
        (vault
            .posts
            .iter()
            .cloned()
            .map(|post| {
                view! {
                    PostCard(post=post)
                }
            })
            .collect::<Vec<_>>())
    }
}

#[component(inline_props)]
pub fn PostCard(post: PostMeta) -> View {
    let summary_html = post.summary.clone();
    let created = OffsetDateTime::from_unix_timestamp(post.created).unwrap();
    let updated = OffsetDateTime::from_unix_timestamp(post.updated).unwrap();

    view! {
        div(
            class="w-full flex flex-col gap-2 p-2 rounded border border-slate-200 hover:border-slate-400"
        ) {
            a(href=format!("/posts/{}", post.id)) {
                h2 { (post.title.clone()) }
            }
            div(class="flex gap-2") {
                span(class="text-xs text-gray-400") {
                    "创建日期: " (format!("{}-{}-{}",
                        created.year(),
                        u8::from(created.month()),
                        created.day()
                    ))
                }
                span(class="text-xs text-gray-400") {
                    "更新日期: " (format!("{}-{}-{}",
                        updated.year(),
                        u8::from(updated.month()),
                        updated.day()
                    ))
                }
            }
            div(class="summary", dangerously_set_inner_html=summary_html)
        }
    }
}

#[component(inline_props)]
pub async fn Post(slug: String) -> View {
    let config = use_context::<ConfigContext>();
    let vault = use_context::<VaultData>();

    let exists_slug = slug.clone();
    let post_meta = vault.posts.iter().find(|p| p.id == exists_slug);

    if post_meta.is_none() {
        navigate("/404");
        return view! {};
    }
    let post_meta = post_meta.unwrap();

    let base_url = config
        .vault_base_url
        .clone()
        .unwrap_or_else(|| "/vault".to_string());

    let fetch_path = post_meta.ids.join("/");
    let article = fetch_article(&base_url, &fetch_path).await;

    let giscus_options = config.giscus_options.clone();

    let content_view = match article {
        Ok(article) => {
            let html = article.content.clone();
            let giscus_opts = giscus_options.clone();
            view! {
                div(class="markdown w-full") {
                    h1 { (article.meta.title.clone()) }
                    div(dangerously_set_inner_html=html)
                }

                (giscus_opts.clone().map(|options| {
                    view! { components::giscus::Giscus(options=options) }
                }))
            }
        }
        Err(e) => {
            let error_msg = format!("Error loading article: {}", e);
            view! { (error_msg) }
        }
    };

    view! {
        Suspense(fallback=move || view! { "Loading article..." }) {
             (content_view)
        }
    }
}

#[component]
pub fn Notes() -> View {
    view! {
        Note(path=vec![])
    }
}

#[component(inline_props)]
pub async fn Note(path: Vec<String>) -> View {
    let config = use_context::<ConfigContext>();
    let vault = use_context::<VaultData>();

    let mut ids = vec!["notes".to_string()];
    ids.extend(path.clone());

    let fetch_path = ids.join("/");

    let base_url = config
        .vault_base_url
        .clone()
        .unwrap_or_else(|| "/vault".to_string());

    let article = fetch_article(&base_url, &fetch_path).await;
    let giscus_options = config.giscus_options.clone();

    let content_view = match article {
        Ok(article) => {
            let html = article.content.clone();
            let giscus_opts = giscus_options.clone();
            view! {
                div(class="markdown w-full") {
                    h1 { (article.meta.title.clone()) }
                    div(dangerously_set_inner_html=html)
                }
                (giscus_opts.clone().map(|options| {
                    view! { components::giscus::Giscus(options=options) }
                }))
            }
        }
        Err(_) => {
            if path.is_empty() {
                view! {
                    div(class="markdown") {
                        h1 { "Notes" }
                        p { "Select a note from the sidebar." }
                    }
                }
            } else {
                view! { "Note not found or no content." }
            }
        }
    };

    let notes_nodes = vault.notes.clone();
    let current_path_clone = ids.clone();

    view! {
        div(class="flex w-full gap-8 items-start") {
            aside(class="w-64 flex-shrink-0 hidden md:block") {
                nav(class="sticky top-4") {
                    NoteTree(nodes=notes_nodes, current_path=current_path_clone)
                }
            }

            div(class="flex-grow min-w-0") {
                Suspense(fallback=move || view! { "Loading note..." }) {
                    (content_view)
                }
            }
        }
    }
}

#[component(inline_props)]
pub fn NoteTree(nodes: Vec<NodeMeta>, current_path: Vec<String>) -> View {
    view! {
        ul(class="flex flex-col gap-1") {
            (nodes.iter().map(|node| {
                let _is_active = current_path.starts_with(&node.ids);
                let is_exact = current_path == node.ids;

                let href = format!("/{}", node.ids.join("/"));
                let title = node.title.clone();
                let children = node.children.clone();
                let next_path = current_path.clone();

                view! {
                    li {
                        div(class=format!("flex items-center gap-1 py-1 px-2 rounded transition-colors {}",
                            if is_exact { "bg-slate-100 font-bold text-primary" } else { "hover:bg-slate-50 text-slate-600" }
                        )) {
                            a(href=href, class="flex-grow truncate block") {
                                (title)
                            }
                        }

                        (if !children.is_empty() {
                            let children = children.clone();
                            let next_path = next_path.clone();
                            view! {
                                div(class="pl-3 border-l border-slate-100 ml-2") {
                                    NoteTree(nodes=children, current_path=next_path)
                                }
                            }
                        } else {
                            view! {}
                        })
                    }
                }
            }).collect::<Vec<_>>())
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
