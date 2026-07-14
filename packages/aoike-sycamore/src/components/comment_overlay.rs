use sycamore::prelude::*;

use crate::components::{render_comment_system, CommentSystem};

#[component(inline_props)]
pub fn CommentOverlay(system: CommentSystem, path: String) -> View {
    let expanded = create_signal(false);

    let toggle = move |_| expanded.set(!expanded.get());
    let collapse = move |_| expanded.set(false);

    view! {
        div(
            class=move || format!(
                "comment-overlay-backdrop {}",
                if expanded.get() { "visible" } else { "" }
            ),
            on:click=collapse
        ) {}
        div(class="comment-overlay") {
            button(
                class="comment-overlay-bar",
                on:click=toggle,
                aria-expanded=move || expanded.get().to_string()
            ) {
                span(class="comment-overlay-title") { "评论" }
                span(class="comment-overlay-chevron") {
                    (move || if expanded.get() { "↓" } else { "↑" })
                }
            }
            div(
                class=move || format!(
                    "comment-overlay-panel {}",
                    if expanded.get() { "expanded" } else { "" }
                )
            ) {
                div(class="comment-overlay-content") {
                    (render_comment_system(
                        &system,
                        if path.is_empty() { None } else { Some(path.clone()) }
                    ))
                }
            }
        }
    }
}
