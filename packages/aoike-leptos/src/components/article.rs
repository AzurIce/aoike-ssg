use leptos::prelude::*;

use crate::{ConfigContext, api::fetch_article};

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
                                view! {
                                    <div class="markdown w-full">
                                        <h1>{article_detail.meta.title.as_str()}</h1>
                                        <div inner_html=article_detail.content.as_str()></div>
                                    </div>
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
