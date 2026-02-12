use aoike::data::VaultMeta;
use leptos::prelude::*;

use crate::{
    layout::tri_column::{Main, TriColumn},
    routes::post::PostCard,
};

#[component]
pub fn Search() -> impl IntoView {
    let vault = use_context::<VaultMeta>().expect("VaultData missing");
    let (query, set_query) = signal(String::new());

    let results = move || {
        let q = query.get().to_lowercase();
        if q.is_empty() {
            return vec![];
        }
        vault
            .posts
            .iter()
            .filter(|p| {
                p.title.to_lowercase().contains(&q)
                    || p.summary.to_lowercase().contains(&q)
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    view! {
        <TriColumn>
            <Main slot>
                <h1>"搜索"</h1>
                <input
                    type="text"
                    placeholder="搜索文章标题或摘要..."
                    class="w-full p-2 border border-slate-300 rounded mb-4"
                    on:input=move |ev| {
                        set_query.set(event_target_value(&ev));
                    }
                    prop:value=query
                />
                {move || {
                    let r = results();
                    if query.get().is_empty() {
                        view! { <p class="text-slate-400">"输入关键词开始搜索"</p> }.into_any()
                    } else if r.is_empty() {
                        view! { <p class="text-slate-400">"没有找到匹配的文章"</p> }.into_any()
                    } else {
                        r.into_iter()
                            .map(|post| view! { <PostCard meta=post /> })
                            .collect_view()
                            .into_any()
                    }
                }}
            </Main>
        </TriColumn>
    }
}
