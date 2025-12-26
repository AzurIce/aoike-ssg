use aoike::data::{ArticleMeta, VaultData};
use leptos::prelude::*;
use leptos_router::{NavigateOptions, components::A, hooks::use_params_map};
use time::OffsetDateTime;

use crate::{ConfigContext, api::fetch_article};

#[component]
pub fn Post() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.get().get("slug").unwrap_or_default();

    let config = use_context::<ConfigContext>().expect("ConfigContext missing");
    let vault = use_context::<VaultData>().expect("VaultData missing");

    let post_meta = move || {
        let s = slug();
        vault.posts.iter().find(|p| p.id == s).cloned()
    };
    let post_url_without_ext = move || post_meta().map(|meta| meta.ids.join("/"));

    // let _p = post_meta.clone();
    let _post_url_without_ext = post_url_without_ext.clone();
    Effect::new(move |_| {
        if _post_url_without_ext().is_none() {
            let navigate = leptos_router::hooks::use_navigate();
            navigate("/4o4", NavigateOptions::default());
        }
    });

    let base_url = config
        .vault_base_url
        .clone()
        .unwrap_or_else(|| "/vault".to_string());

    let article_resource = LocalResource::new(move || {
        let base_url = base_url.clone();
        let post_url_without_ext = post_url_without_ext.clone();
        async move {
            if let Some(p) = post_url_without_ext() {
                fetch_article(&base_url, &p).await.ok()
            } else {
                let navigate = leptos_router::hooks::use_navigate();
                navigate("/4o4", NavigateOptions::default());
                None
            }
        }
    });

    let giscus_options = config.giscus_options.clone();

    view! {
        <Suspense fallback=move || {
            view! { "Loading article..." }
        }>
            {move || {
                article_resource
                    .map(|article| {
                        match article {
                            Some(article) => {
                                let giscus_opts = giscus_options.clone();
                                // let html = article.content.clone();
                                view! {
                                    <div class="markdown w-full">
                                        <h1>{article.meta.title.as_str()}</h1>
                                        <div inner_html=article.content.as_str()></div>
                                    </div>
                                    {giscus_opts
                                        .map(|options| {
                                            view! {
                                                <crate::components::giscus::Giscus options=options />
                                            }
                                        })}
                                }
                                    .into_any()
                            }
                            None => view! { "Loading..." }.into_any(),
                        }
                    })
            }}
        </Suspense>
    }
}

#[component]
pub fn Posts() -> impl IntoView {
    let vault = use_context::<VaultData>().expect("VaultData missing");
    view! {
        <h1>"所有文章"</h1>
        {vault
            .posts
            .into_iter()
            .map(|post| {
                view! { <PostCard meta=post /> }
            })
            .collect_view()}
    }
}

#[component]
pub fn PostCard(meta: ArticleMeta) -> impl IntoView {
    let summary_html = meta.summary.clone();
    let created = OffsetDateTime::from_unix_timestamp(meta.created).unwrap();
    let updated = OffsetDateTime::from_unix_timestamp(meta.updated).unwrap();

    view! {
        <div class="w-full flex flex-col gap-2 p-2 rounded border border-slate-200 hover:border-slate-400">
            <A href=format!("/posts/{}", meta.id)>
                <h2>{meta.title}</h2>
            </A>
            <div class="flex gap-2">
                <span class="text-xs text-gray-400">
                    "创建日期: "
                    {format!("{}-{}-{}", created.year(), u8::from(created.month()), created.day())}
                </span>
                <span class="text-xs text-gray-400">
                    "更新日期: "
                    {format!("{}-{}-{}", updated.year(), u8::from(updated.month()), updated.day())}
                </span>
            </div>
            <div class="summary" inner_html=summary_html></div>
        </div>
    }
}
