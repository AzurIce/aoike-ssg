use aoike::data::{ArticleMeta, NodeMeta, SectionMeta, VaultData};
use leptos::{
    leptos_dom::logging::{console_debug_log, console_log},
    prelude::*,
};
use leptos_router::{NavigateOptions, components::A, hooks::use_params_map};

use crate::{
    BASE_URL,
    components::article::Article,
    layout::tri_column::{Left, Main, TriColumn},
};

#[component]
pub fn Notes() -> impl IntoView {
    let vault = use_context::<VaultData>().expect("VaultData missing");

    view! {
        <TriColumn>
            <Main slot>
                <div class="flex gap-2 flex-wrap">
                    {vault
                        .notes
                        .iter()
                        .cloned()
                        .map(|note| {
                            let href = format!("{BASE_URL}/{}", note.ids.join("/"));
                            console_debug_log(&format!("{:?}", note));
                            view! {
                                <div class="flex items-center p-2 rounded border border-slate-200 hover:border-slate-400 hover:bg-gray-100">
                                    <A href={href} {..} class="flex-grow truncate block">
                                        {note.title.clone()}
                                    </A>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </Main>
        </TriColumn>
    }
}

#[component]
pub fn Note() -> impl IntoView {
    let params = use_params_map();
    let path = move || params.read().get("path");
    let vault = use_context::<VaultData>().expect("failed to get vault data");
    let note_meta = Memo::new(move |_| {
        path()
            .and_then(|p| p.split("/").next().map(|s| s.to_string()))
            .and_then(|id| vault.notes.iter().find(|n| n.id == id).cloned())
    });

    let article_url_without_ext = move || path().map(|p| format!("/notes/{p}"));

    Effect::new(move || {
        if note_meta().is_none() {
            let navigate = leptos_router::hooks::use_navigate();
            navigate(&format!("{BASE_URL}/4o4"), NavigateOptions::default());
        }
    });

    view! {
        <TriColumn>
            <Left slot>
                {move || {
                    note_meta()
                        .map(|note_meta| {
                            view! {
                                <nav class="sticky top-4">
                                    <NoteTree section=note_meta.clone() />
                                </nav>
                            }
                        })
                }}
            </Left>
            <Main slot>
                {move || {
                    article_url_without_ext()
                        .map(|ids_path| {
                            view! {
                                <Article
                                    ids_path=move || ids_path.clone()
                                    on_failed=|err| {
                                        console_log(&format!("Failed to fetch article: {:?}", err));
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
pub fn NoteTreeArticleNode(article: ArticleMeta) -> impl IntoView {
    let params = use_params_map();
    let path = move || params.read().get("path").clone();

    let href = format!("{BASE_URL}/{}", article.ids.join("/"));
    let title = article.title.clone();
    let ids = article.ids.clone();

    let is_active = move || path().map(|p| format!("notes/{}", p)) == Some(ids.join("/"));
    view! {
        <li>
            <div class=move || {
                if is_active() {
                    "flex items-center gap-1 py-1 px-2 rounded transition-colors bg-slate-100 font-bold text-primary"
                } else {
                    "flex items-center gap-1 py-1 px-2 rounded transition-colors hover:bg-slate-50 text-slate-600"
                }
            }>
                <A href={href} {..} class="flex-grow truncate block">
                    {title}
                </A>
            </div>
        </li>
    }
}

#[component]
pub fn NoteTreeSectionNode(section: SectionMeta) -> impl IntoView {
    let params = use_params_map();
    let path = move || params.read().get("path").clone();

    let href = format!("{BASE_URL}/{}", section.ids.join("/"));
    let title = section.title.clone();
    let children = section.children.clone();
    let ids = section.ids.clone();

    let is_active = move || path().map(|p| format!("notes/{}", p)) == Some(ids.join("/"));

    view! {
        <li>
            <div class=move || {
                if is_active() {
                    "flex items-center gap-1 py-1 px-2 rounded transition-colors bg-slate-100 font-bold text-primary"
                } else {
                    "flex items-center gap-1 py-1 px-2 rounded transition-colors hover:bg-slate-50 text-slate-600"
                }
            }>
                <A href={href} {..} class="flex-grow truncate block">
                    {title}
                </A>
            </div>
            {if !children.is_empty() {
                view! {
                    <ul class="pl-3 border-l border-slate-100 ml-2 flex flex-col gap-1 mt-1">
                        {children
                            .into_iter()
                            .map(|child| {
                                view! { <NoteTreeNode node=child /> }
                            })
                            .collect_view()}
                    </ul>
                }
                    .into_any()
            } else {
                view! {}.into_any()
            }}
        </li>
    }
}

#[component]
pub fn NoteTreeNode(node: NodeMeta) -> impl IntoView {
    match node {
        NodeMeta::Article(article) => view! {
            <NoteTreeArticleNode article=article />
        }
        .into_any(),
        NodeMeta::Section(section) => view! {
            <NoteTreeSectionNode section=section />
        }
        .into_any(),
    }
}

/// The Tree of the nodes in a note.
#[component]
pub fn NoteTree(section: SectionMeta) -> impl IntoView {
    let href = format!("{BASE_URL}/{}", section.ids.join("/"));

    view! {
        <div class="mb-2 px-2 flex items-center gap-2">
            <A
                href="../"
                {..}
                class="text-slate-500 hover:text-slate-700 transition-colors"
                title="Back to Notes"
            >
                <div class="i-mdi-arrow-left text-xl" />
            </A>
            <A href={href} {..} class="font-bold text-lg hover:underline">
                {section.title.clone()}
            </A>
        </div>
        <ul class="flex flex-col gap-1">
            {section
                .children
                .into_iter()
                .map(|child| {
                    view! { <NoteTreeNode node=child /> }
                })
                .collect_view()}
        </ul>
    }
}
