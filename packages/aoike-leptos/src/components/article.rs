use leptos::prelude::*;
use leptos_router::components::A;

use crate::{ConfigContext, api::fetch_article, utils::based_url};

#[component]
pub fn Article(
    ids_path: impl Fn() -> String + 'static,
    on_failed: impl Fn(gloo_net::Error) + Clone + 'static,
) -> impl IntoView {
    let config = use_context::<ConfigContext>().expect("ConfigContext missing");
    let base_url = config
        .vault_base_url
        .clone()
        .unwrap_or_else(|| "/vault".to_string());

    let article_detail = LocalResource::new(move || {
        let base_url = base_url.clone();
        let ids_path = ids_path();
        let on_failed = on_failed.clone();
        async move {
            match fetch_article(&base_url, &ids_path).await {
                Ok(article) => Some(article),
                Err(err) => {
                    on_failed(err);
                    None
                }
            }
        }
    });

    view! {
        <Suspense fallback=move || {
            view! { "Loading article..." }
        }>
            {move || {
                article_detail
                    .map(|article_detail| {
                        match article_detail {
                            Some(article_detail) => {
                                let backlinks = article_detail.backlinks.clone();
                                view! {
                                    <div class="markdown w-full">
                                        <h1>{article_detail.meta.title.as_str()}</h1>
                                        <div inner_html=article_detail.content.as_str()></div>
                                    </div>
                                    {if !backlinks.is_empty() {
                                        Some(view! {
                                            <div class="mt-8 pt-4 border-t border-slate-200">
                                                <h3 class="text-sm font-medium text-slate-500 mb-2">
                                                    "反向链接"
                                                </h3>
                                                <ul class="flex flex-col gap-1">
                                                    {backlinks
                                                        .into_iter()
                                                        .map(|link| {
                                                            let link_url = based_url(format!("notes/{}", &link));
                                                            let display = link.split('/').last().unwrap_or(&link).to_string();
                                                            view! {
                                                                <li>
                                                                    <A href={link_url} {..} class="text-sm text-blue-600 hover:underline">
                                                                        {display}
                                                                    </A>
                                                                </li>
                                                            }
                                                        })
                                                        .collect_view()}
                                                </ul>
                                            </div>
                                        })
                                    } else {
                                        None
                                    }}
                                    {config
                                        .giscus_options
                                        .clone()
                                        .map(|options| {
                                            view! {
                                                <crate::components::giscus::Giscus options=options />
                                            }
                                        })}
                                }
                                    .into_any()
                            }
                            None => view! { "Article not found" }.into_any(),
                        }
                    })
            }}
        </Suspense>
    }
}
