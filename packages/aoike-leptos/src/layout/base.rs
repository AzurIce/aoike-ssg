use crate::utils::based_url;
use leptos::prelude::*;
use leptos_router::components::A;

use crate::ConfigContext;

#[component]
pub fn Header() -> impl IntoView {
    let config = use_context::<ConfigContext>().expect("ConfigContext not provided");

    let title = config.title.clone().unwrap_or("Site Title".to_string());
    let desc = config
        .desc
        .clone()
        .unwrap_or("site description".to_string());

    view! {
        <header class="flex sticky top-0 w-full bg-transparent z-800">
            <div class="absolute size-full z-[-1] border-b border-b-slate-300 bg-white/90 backdrop-blur-md"></div>
            <nav class="flex gap-2 items-center p-x-6 max-w-5xl h-14 w-full m-x-auto">
                <A
                    href=based_url("/")
                    {..}
                    class="flex gap-2 m-r-auto nav-btn h-10 p-1 group"
                >
                    {config
                        .avatar
                        .clone()
                        .map(|avatar| {
                            let avatar = based_url(format!(
                                "static/{}",
                                avatar.trim_start_matches("/"),
                            ));
                            view! { <img class="h-full rounded" src=avatar alt="avatar" /> }
                        })}
                    <div class="flex flex-col">
                        <span class="text-sm transition-transform duration-500 group-hover:-translate-y-1">
                            {title}
                        </span>
                        <span class="text-xs text-slate-600 opacity-0 max-h-0 overflow-hidden transition-all duration-500 group-hover:opacity-100 group-hover:max-h-8">
                            {desc}
                        </span>
                    </div>
                </A>
                <A href=based_url("posts") {..} class="h-10 gap-1 nav-btn text-sm p-x-4">
                    "文章"
                </A>
                <A href=based_url("notes") {..} class="h-10 gap-1 nav-btn text-sm p-x-4">
                    "笔记"
                </A>
                <A href=based_url("search") {..} class="h-10 gap-1 nav-btn text-sm p-x-4">
                    "搜索"
                </A>
                {config
                    .github_owner
                    .clone()
                    .zip(config.github_repo.clone())
                    .map(|(owner, repo)| {
                        view! {
                            <a
                                class="size-10 gap-1 nav-btn"
                                href=format!("https://github.com/{}/{}", owner, repo)
                                rel="noreferrer"
                            >
                                <div class="i-fa6-brands-github text-2xl"></div>
                            </a>
                        }
                    })}
            </nav>
        </header>
    }
}
