use aoike::data::{ArticleMeta, VaultMeta};
use leptos::{leptos_dom::logging::console_debug_log, prelude::*};
use leptos_router::{NavigateOptions, components::A, hooks::use_params_map};
use time::OffsetDateTime;

use crate::{
    components::article::Article,
    layout::tri_column::{Main, TriColumn},
    utils::based_url,
};

#[component]
pub fn Post() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("slug").unwrap_or_default();

    let vault = use_context::<VaultMeta>().expect("VaultData missing");

    let post_meta = move || {
        let s = slug();
        vault
            .posts
            .iter()
            .find(|p| p.entity_path.id() == Some(&s))
            .cloned()
    };
    let post_ids_path = move || post_meta().map(|meta| meta.entity_path.ids_path());

    view! {
        <TriColumn>
            <Main slot>
                {move || {
                    post_ids_path()
                        .map(|ids_path| {
                            view! {
                                <Article
                                    ids_path=move || ids_path.clone()
                                    on_failed=|err| {
                                        let navigate = leptos_router::hooks::use_navigate();
                                        navigate(
                                            &based_url("4o4"),
                                            NavigateOptions::default(),
                                        );
                                        console_debug_log(&format!("{err:?}"));
                                    }
                                />
                            }
                        })
                }}
            </Main>
        </TriColumn>
    }
}

#[component]
pub fn Posts() -> impl IntoView {
    let vault = use_context::<VaultMeta>().expect("VaultData missing");
    view! {
        <TriColumn>
            <Main slot>
                <h1>"所有文章"</h1>
                {vault
                    .posts
                    .into_iter()
                    .map(|post| {
                        view! { <PostCard meta=post /> }
                    })
                    .collect_view()}
            </Main>
        </TriColumn>
    }
}

#[component]
pub fn PostCard(meta: ArticleMeta) -> impl IntoView {
    let summary_html = meta.summary.clone();
    let tags = meta.tags.clone();
    let created = OffsetDateTime::from_unix_timestamp(meta.created).unwrap();
    let updated = OffsetDateTime::from_unix_timestamp(meta.updated).unwrap();

    view! {
        <div class="w-full flex flex-col gap-2 p-2 rounded border border-slate-200 hover:border-slate-400">
            <A href=based_url(format!("posts/{}", meta.entity_path.id().unwrap()))>
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
            {if !tags.is_empty() {
                Some(view! {
                    <div class="flex gap-1 flex-wrap">
                        {tags
                            .into_iter()
                            .map(|tag| {
                                let tag_href = based_url(format!("tags/{}", tag));
                                view! {
                                    <A href={tag_href} {..} class="text-xs px-2 py-0.5 rounded-full bg-slate-100 text-slate-600 hover:bg-slate-200">
                                        {tag}
                                    </A>
                                }
                            })
                            .collect_view()}
                    </div>
                })
            } else {
                None
            }}
            <div class="summary" inner_html=summary_html></div>
        </div>
    }
}

/// Tags list page — shows all unique tags with article counts.
#[component]
pub fn Tags() -> impl IntoView {
    let vault = use_context::<VaultMeta>().expect("VaultData missing");

    let mut tag_counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for post in &vault.posts {
        for tag in &post.tags {
            *tag_counts.entry(tag.clone()).or_default() += 1;
        }
    }

    view! {
        <TriColumn>
            <Main slot>
                <h1>"标签"</h1>
                <div class="flex gap-2 flex-wrap">
                    {tag_counts
                        .into_iter()
                        .map(|(tag, count)| {
                            let tag_href = based_url(format!("tags/{}", tag));
                            view! {
                                <A href={tag_href} {..} class="px-3 py-1 rounded-full bg-slate-100 text-slate-700 hover:bg-slate-200">
                                    {format!("{} ({})", tag, count)}
                                </A>
                            }
                        })
                        .collect_view()}
                </div>
            </Main>
        </TriColumn>
    }
}

/// Tag detail page — shows articles filtered by a specific tag.
#[component]
pub fn Tag() -> impl IntoView {
    let params = use_params_map();
    let tag = move || params.read().get("tag").unwrap_or_default();

    let vault = use_context::<VaultMeta>().expect("VaultData missing");

    view! {
        <TriColumn>
            <Main slot>
                {move || {
                    let current_tag = tag();
                    let filtered: Vec<_> = vault
                        .posts
                        .iter()
                        .filter(|p| p.tags.contains(&current_tag))
                        .cloned()
                        .collect();
                    view! {
                        <h1>{format!("标签: {}", current_tag)}</h1>
                        {filtered
                            .into_iter()
                            .map(|post| {
                                view! { <PostCard meta=post /> }
                            })
                            .collect_view()}
                    }
                }}
            </Main>
        </TriColumn>
    }
}
