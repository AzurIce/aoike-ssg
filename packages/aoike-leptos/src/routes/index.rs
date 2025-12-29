use aoike::data::VaultData;
use leptos::prelude::*;
use leptos_router::components::A;
use time::OffsetDateTime;

use crate::{
    ConfigContext,
    api::fetch_article,
    layout::tri_column::{Main, TriColumn},
};

#[component]
pub fn Index() -> impl IntoView {
    let config = use_context::<ConfigContext>().expect("ConfigContext missing");
    let vault = use_context::<VaultData>().expect("VaultData missing");

    let base_url = config
        .vault_base_url
        .clone()
        .unwrap_or_else(|| "/vault".to_string());

    let index_article_resource = LocalResource::new(move || {
        let base_url = base_url.clone();
        async move { fetch_article(&base_url, "posts").await.ok() }
    });

    let recent_posts = vault.posts.iter().take(5).cloned().collect::<Vec<_>>();

    view! {
        <TriColumn>
            <Main slot>
                <Hero />

                <div class="flex flex-col w-full p-2 markdown">
                    <h2>"最新文章"</h2>
                    <ul>
                        {recent_posts
                            .into_iter()
                            .map(|blog| {
                                let created = OffsetDateTime::from_unix_timestamp(blog.created)
                                    .unwrap();
                                view! {
                                    <li class="flex gap-8 items-baseline">
                                        <span class="text-gray-600 text-sm font-mono">
                                            {format!(
                                                "{}-{:02}-{:02}",
                                                created.year(),
                                                u8::from(created.month()),
                                                created.day(),
                                            )}
                                        </span>
                                        <A
                                            href=format!("/posts/{}", blog.id)
                                            {..}
                                            class="underline hover:underline-gray-400"
                                        >
                                            {blog.title}
                                        </A>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                    <hr />
                    <Suspense fallback=move || {
                        view! { "Loading content..." }
                    }>
                        {move || {
                            index_article_resource
                                .get()
                                .map(|res| {
                                    match res {
                                        Some(article) => {
                                            view! { <div inner_html=article.content></div> }.into_any()
                                        }
                                        None => view! {}.into_any(),
                                    }
                                })
                        }}
                    </Suspense>
                </div>

                {move || {
                    config
                        .giscus_options
                        .clone()
                        .map(|options| {
                            view! { <crate::components::giscus::Giscus options=options /> }
                        })
                }}
            </Main>
        </TriColumn>
    }
}

#[component]
pub fn Hero() -> impl IntoView {
    let config = use_context::<ConfigContext>().expect("ConfigContext missing");

    let title = config.title.clone().unwrap_or("Site Title".to_string());
    let desc = config
        .desc
        .clone()
        .unwrap_or("site description".to_string());

    view! {
        <div class="flex items-stretch">
            {config
                .avatar
                .clone()
                .map(|avatar| {
                    view! { <img class="size-40 rounded" src=avatar /> }
                })} <div class="flex flex-col items-center justify-around p-2 p-b-1 gap-3">
                <span class="text-xl lxgw">"< " {title} " />"</span>

                <span class="text-sm lxgw">{desc}</span>

                {config
                    .email
                    .clone()
                    .map(|email| {
                        let _email = email.clone();
                        view! {
                            <span class="text-sm">
                                "📫 " <a class="underline" href=format!("mailto:{}", email)>
                                    {_email}
                                </a>
                            </span>
                        }
                    })}

                <div class="flex">
                    {config
                        .github_owner
                        .clone()
                        .map(|owner| {
                            view! {
                                <A
                                    href=format!("https://github.com/{}", owner)
                                    target="_blank"
                                    {..}
                                    class="size-8 gap-1 nav-btn"
                                >
                                    <div class="i-fa6-brands-github text-xl"></div>
                                </A>
                            }
                        })}
                    {config
                        .bilibili_url
                        .clone()
                        .map(|url| {
                            view! {
                                <a
                                    href=url
                                    target="_blank"
                                    rel="noreferrer"
                                    class="size-8 gap-1 nav-btn"
                                >
                                    <div class="i-fa6-brands-bilibili text-xl color-[#19a2d4] translate-x-0 translate-y-[1px]"></div>
                                </a>
                            }
                        })}
                    {config
                        .steam_url
                        .clone()
                        .map(|url| {
                            view! {
                                <a
                                    href=url
                                    target="_blank"
                                    rel="noreferrer"
                                    class="size-8 gap-1 nav-btn"
                                >
                                    <div class="i-fa6-brands-steam text-xl bg-[#082256]"></div>
                                </a>
                            }
                        })}
                </div>
            </div>
        </div>
    }
}
