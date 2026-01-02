use aoike::data::{NodeMeta, SectionMeta, VaultMeta};
use enclose::enclose;
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
    let vault = use_context::<VaultMeta>().expect("VaultData missing");

    view! {
        <TriColumn>
            <Main slot>
                <div class="flex gap-2 flex-wrap">
                    {vault
                        .notes
                        .iter()
                        .cloned()
                        .map(|note| {
                            let href = format!("{BASE_URL}/{}", note.entity_path.ids_path());
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
    let vault = use_context::<VaultMeta>().expect("failed to get vault data");
    let note_meta = Memo::new(move |_| {
        path()
            .and_then(|p| p.split("/").next().map(|s| s.to_string()))
            .and_then(|id| {
                vault
                    .notes
                    .iter()
                    .find(|n| n.entity_path.id() == Some(&id))
                    .cloned()
            })
    });
    let current_node = RwSignal::<Option<NodeMeta>>::new(None);
    Effect::new(move || {
        if current_node.read().is_none() {
            current_node.set(note_meta.get().and_then(|section| {
                section
                    .index
                    .map(NodeMeta::Article)
                    .or(section.children.first().cloned())
            }));
        }
    });

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
                                    <NoteTree
                                        root_section=note_meta.clone()
                                        current_node=current_node
                                    />
                                </nav>
                            }
                        })
                }}
            </Left>
            <Main slot>
                {move || {
                    current_node
                        .get()
                        .map(|node| {
                            view! {
                                <Article
                                    ids_path=move || node.entity_path().ids_path()
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
pub fn NoteTreeNode(node: NodeMeta, current_node: RwSignal<Option<NodeMeta>>) -> impl IntoView {
    let title = match &node {
        NodeMeta::Article(article) => article.title.clone(),
        NodeMeta::Section(section) => section.title.clone(),
    };
    let href = match &node {
        NodeMeta::Article(article) => format!("{BASE_URL}/{}", article.entity_path.ids_path()),
        NodeMeta::Section(section) => format!("{BASE_URL}/{}", section.entity_path.ids_path()),
    };
    let active = enclose!((node) move || {
        current_node.get().map(|n|
            n.entity_path().ids_path() == node.entity_path().ids_path()
        ).unwrap_or_default()
    });
    let not_active = enclose!((active) move || !active());

    view! {
        <li>
            <div
                class=(
                    ["bg-slate-100", "font-bold", "text-primary"],
                    enclose!((active) move || active()),
                )
                class=(
                    ["hover:bg-slate-50", "text-slate-600"],
                    enclose!((not_active) move || not_active()),
                )
                class="flex items-center gap-1 py-1 px-2 rounded transition-colors"
            >
                <A
                    href=href
                    on:click={enclose!((node) move |_| current_node.set(Some(node.clone())))}
                    {..}
                    class="flex-grow truncate block"
                >
                    {title}
                </A>
            </div>
        </li>
        {enclose!(
            (node) move ||
            match node.clone() {
                NodeMeta::Article(_) => {
                    view! {}.into_any()
                }
                NodeMeta::Section(section) => {
                    view! {
                        <ul class="pl-3 border-l border-slate-100 ml-2 flex flex-col gap-1 mt-1">
                            {section.children
                                .into_iter()
                                .map(move |child| {
                                    view! { <NoteTreeNode node=child current_node /> }
                                })
                                .collect_view()}
                        </ul>
                    }.into_any()
                }
            }
        )}
    }
}

/// The Tree of the nodes in a note.
#[component]
pub fn NoteTree(
    root_section: SectionMeta,
    current_node: RwSignal<Option<NodeMeta>>,
) -> impl IntoView {
    let href = format!("{BASE_URL}/{}", root_section.entity_path.ids_path());
    let first_child = root_section.children.first().cloned();
    let index = root_section.index.map(NodeMeta::Article);

    // Effect::new(move || {
    //     if current_node.read().is_none() {
    //         current_node.set(index.clone().or(first_child.clone()));
    //     }
    // });

    view! {
        // TODO: Bradcrumb
        // <div class="flex">
        // {move || current_node.get().is_none()}
        // {move || current_node.get().map(|x| {
        // x.entity_path().ids_path()
        // })}
        // </div>
        <div class="mb-2 px-2 flex items-center gap-2">
            <A
                href=format!("{BASE_URL}/notes")
                {..}
                class="text-slate-500 hover:text-slate-700 transition-colors"
                title="Back to Notes"
            >
                <div class="i-mdi-arrow-left text-xl" />
            </A>
            // TODO: only clickable if index is some
            <A
                href=href
                on:click={move |_| current_node.set(index.clone().or(first_child.clone()))}
                {..}
                class="font-bold text-lg hover:underline"
            >
                {root_section.title.clone()}
            </A>
        </div>
        <ul class="flex flex-col gap-1">
            {root_section
                .children
                .into_iter()
                .map(|child| {
                    view! { <NoteTreeNode node=child current_node=current_node /> }
                })
                .collect_view()}
        </ul>
    }
}
