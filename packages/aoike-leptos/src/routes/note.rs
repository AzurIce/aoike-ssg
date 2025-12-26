use aoike::data::{NodeMeta, VaultData};
use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_params_map};

use crate::{ConfigContext, api::fetch_article};

#[component]
pub fn Notes() -> impl IntoView {
    view! { <Note /> }
}

#[component]
pub fn Note() -> impl IntoView {
    let params = use_params_map();
    let path_param = move || params.get().get("path").unwrap_or_default();

    let config = use_context::<ConfigContext>().expect("ConfigContext missing");
    let vault = use_context::<VaultData>().expect("VaultData missing");

    let ids = move || {
        let p = path_param();
        let mut parts = vec!["notes".to_string()];
        if !p.is_empty() {
            parts.extend(p.split('/').map(|s| s.to_string()));
        }
        parts
    };

    let base_url = config
        .vault_base_url
        .clone()
        .unwrap_or_else(|| "/vault".to_string());

    let article_resource = Resource::new(
        move || ids(),
        move |ids| {
            let base_url = base_url.clone();
            let fetch_path = ids.join("/");
            async move { fetch_article(&base_url, &fetch_path).await.ok() }
        },
    );

    let giscus_options = config.giscus_options.clone();
    let notes_nodes = vault.notes.clone();

    view! {
        <div class="flex w-full gap-8 items-start">
            <aside class="w-64 flex-shrink-0 hidden md:block">
                <nav class="sticky top-4">
                    <NoteTree nodes=notes_nodes current_path=ids />
                </nav>
            </aside>

            <div class="flex-grow min-w-0">
                <Suspense fallback=move || {
                    view! { "Loading note..." }
                }>// {move || {
                // article_resource.get().map(|article| {

                // match article {
                // Some(article) => {
                // let html = article.content.clone();
                // let giscus_opts = giscus_options.clone();
                // view! {
                // <div class="markdown w-full">
                // <h1>{article.meta.title}</h1>
                // <div inner_html=html></div>
                // </div>
                // {giscus_opts
                // .map(|options| {
                // view! { <crate::components::giscus::Giscus options=options /> }
                // })}
                // }
                // .into_any()
                // }
                // None => {
                // if path_param().is_empty() {
                // view! {
                // <div class="markdown">
                // <h1>"Notes"</h1>
                // <p>"Select a note from the sidebar."</p>
                // </div>
                // }
                // .into_any()
                // } else {
                // view! { "Note not found or no content." }.into_any()
                // }
                // }
                // None => view! { "Loading..." }.into_any(),
                // }
                // })
                // }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
pub fn NoteTree(
    nodes: Vec<NodeMeta>,
    #[prop(into)] current_path: Signal<Vec<String>>,
) -> impl IntoView {
    view! {
        <ul class="flex flex-col gap-1">
            {nodes
                .into_iter()
                .map(|node| {
                    let node_ids = node.ids.clone();
                    let is_exact = move || current_path.get() == node_ids;
                    let href = format!("/{}", node.ids.join("/"));
                    let title = node.title.clone();
                    let children = node.children.clone();

                    view! {
                        <li>
                            <div class=move || {
                                format!(
                                    "flex items-center gap-1 py-1 px-2 rounded transition-colors {}",
                                    if is_exact() {
                                        "bg-slate-100 font-bold text-primary"
                                    } else {
                                        "hover:bg-slate-50 text-slate-600"
                                    },
                                )
                            }>
                                <A href=href>{title}</A>
                            </div>

                            {if !children.is_empty() {
                                view! {
                                    <div class="pl-3 border-l border-slate-100 ml-2">
                                        <NoteTree nodes=children current_path=current_path />
                                    </div>
                                }
                                    .into_any()
                            } else {
                                view! {}.into_any()
                            }}
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
}
