#[cfg(feature = "build")]
pub mod build;

pub mod docsgen;

use std::rc::Rc;

use aoike::{GalleryCategory, GalleryImage, PostData};
use sycamore::prelude::*;
use sycamore_router::{navigate, HistoryIntegration, Route, Router};

pub mod components {
    pub mod giscus;
}

use crate::{components::giscus::GiscusOptions, layout::base::Header};

pub mod layout {
    pub mod base;
}

#[derive(Route, Clone)]
enum AppRoutes {
    #[to("/")]
    Index,
    #[to("/posts")]
    Posts,
    #[to("/posts/<slug>")]
    Post { slug: String },
    #[to("/gallery")]
    Gallery,
    #[not_found]
    NotFound,
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct ConfigContext {
    pub title: Option<String>,
    pub desc: Option<String>,
    pub email: Option<String>,
    pub avatar: Option<String>,
    pub github_owner: Option<String>,
    pub github_repo: Option<String>,
    pub bilibili_url: Option<String>,
    pub steam_url: Option<String>,
    // pub extra_head: Option<<dyn FnOnce() -> View>>,
    pub giscus_options: Option<GiscusOptions>,
}

#[component(inline_props)]
pub fn AoikeApp(
    config: ConfigContext,
    index: &'static PostData,
    posts: &'static [PostData],
    gallery: &'static [GalleryCategory],
) -> View {
    provide_context(config);

    view! {
        Router(
            integration=HistoryIntegration::new(),
            view=move |route: ReadSignal<AppRoutes>| {
                view! {
                    Header()

                    main(class="max-w-[80ch] w-full m-x-auto flex flex-col items-center p-8 gap-4") {
                        (match route.get_clone() {
                            AppRoutes::Index => view! {
                                Index(index=index, posts=posts)
                            },
                            AppRoutes::Posts => view! {
                                Posts(posts=posts)
                            },
                            AppRoutes::Post { slug } => view! {
                                Post(posts=posts, slug=slug)
                            },
                            AppRoutes::Gallery => view! {
                                GalleryPage(categories=gallery)
                            },
                            AppRoutes::NotFound => view! {
                                NotFound()
                            },
                        })
                    }
                }
            }
        )
    }
}

#[component(inline_props)]
pub fn Index(index: &'static PostData, posts: &'static [PostData]) -> View {
    let config = use_context::<ConfigContext>();

    let recent_posts_view = posts
        .iter()
        .take(5)
        .map(|blog| {
            view! {
                li(class="flex gap-8") {
                    span(class="text-gray-600") {
                        (format!("{}-{}-{}",
                            blog.created.year(),
                            u8::from(blog.created.month()),
                            blog.created.day()
                        ))
                    }
                    a(
                        class="underline hover:underline-gray-400",
                        href=format!("/posts/{}", blog.slug)
                    ) {
                        (blog.title.clone())
                    }
                }
            }
        })
        .collect::<Vec<View>>();

    let content_html = index.content_html.as_str();

    view! {
        Hero()

        div(class="flex flex-col w-full p-2 markdown") {
            h2 { "最新文章" }
            ul {
                (recent_posts_view)
            }
            hr {}
            div(dangerously_set_inner_html=content_html)
        }

        (config.giscus_options.clone().map(|options| {
            view! { components::giscus::Giscus(options=options) }
        }))
    }
}

#[component]
pub fn Hero() -> View {
    let config = use_context::<ConfigContext>();

    let title = config.title.as_deref().unwrap_or("Site Title").to_string();
    let desc = config
        .desc
        .as_deref()
        .unwrap_or("site description")
        .to_string();

    view! {
        div(class="flex items-stretch") {
            (config.avatar.clone().map(|avatar| {
                view! {
                    img(class="size-40 rounded", src=avatar)
                }
            }))

            div(class="flex flex-col items-center justify-around p-2 p-b-1 gap-3") {
                span(class="text-xl lxgw") {
                    "< " (title) " />"
                }

                span(class="text-sm lxgw") {
                    (desc)
                }

                (config.email.clone().map(|email| {
                    let _email = email.clone();
                    view! {
                        span(class="text-sm") {
                            "📫 "
                            a(class="underline", href=format!("mailto:{}", email)) {
                                (_email)
                            }
                        }
                    }
                }))

                div(class="flex") {
                    (config.github_owner.clone().map(|owner| {
                        view! {
                            a(href=format!("https://github.com/{}", owner), target="_blank", rel="noreferrer", class="size-8 gap-1 nav-btn") {
                                div(class="i-fa6-brands-github text-xl")
                            }
                        }
                    }))

                    (config.bilibili_url.clone().map(|url| {
                        view! {
                            a(href=url, target="_blank", rel="noreferrer", class="size-8 gap-1 nav-btn") {
                                div(class="i-fa6-brands-bilibili text-xl color-[#19a2d4] translate-x-0 translate-y-[1px]")
                            }
                        }
                    }))

                    (config.steam_url.clone().map(|url| {
                        view! {
                            a(href=url, target="_blank", rel="noreferrer", class="size-8 gap-1 nav-btn") {
                                div(class="i-fa6-brands-steam text-xl bg-[#082256]")
                            }
                        }
                    }))
                }
            }
        }
    }
}

#[component(inline_props)]
pub fn Posts(posts: &'static [PostData]) -> View {
    view! {
        h1 { "所有文章" }
        (posts.iter().map(|post| {
            view! {
                PostCard(post=post)
            }
        }).collect::<Vec<_>>())
    }
}

#[component(inline_props)]
pub fn PostCard(post: &'static PostData) -> View {
    let summary_html = post.summary_html.as_str();
    view! {
        div(
            class="w-full flex flex-col gap-2 p-2 rounded border border-slate-200 hover:border-slate-400"
        ) {
            a(href=format!("/posts/{}", post.slug)) {
                h2 { (post.title.clone()) }
            }
            div(class="flex gap-2") {
                span(class="text-xs text-gray-400") {
                    "创建日期: " (format!("{}-{}-{}",
                        post.created.year(),
                        u8::from(post.created.month()),
                        post.created.day()
                    ))
                }
                span(class="text-xs text-gray-400") {
                    "更新日期: " (format!("{}-{}-{}",
                        post.updated.year(),
                        u8::from(post.updated.month()),
                        post.updated.day()
                    ))
                }
            }
            div(class="summary", dangerously_set_inner_html=summary_html)
        }
    }
}

#[component(inline_props)]
pub fn Post(posts: &'static [PostData], slug: String) -> View {
    let config = use_context::<ConfigContext>();

    let Some(post) = posts.iter().find(|p| p.slug == slug) else {
        navigate("/404");
        return view! {};
    };

    let content_html = post.content_html.as_str();
    view! {
        div(class="markdown w-full") {
            div(dangerously_set_inner_html=content_html)
        }

        (config.giscus_options.clone().map(|options| {
            view! { components::giscus::Giscus(options=options) }
        }))
    }
}

#[component]
pub fn NotFound() -> View {
    view! {
        h1 { "404 Not Found" }
        p { "The page you're looking for doesn't exist." }
    }
}

#[component(inline_props)]
pub fn GalleryPage(categories: &'static [GalleryCategory]) -> View {
    view! {
        div(class="w-full flex flex-col gap-8") {
            h1 { "Gallery" }
            (if categories.is_empty() {
                view! {
                    p(class="text-gray-500") { "还没有图片哦~" }
                }
            } else {
                view! {
                    (categories
                        .iter()
                        .map(|category| {
                            view! {
                                GalleryCategory(category=category)
                            }
                        })
                        .collect::<Vec<_>>())
                }
            })
        }
    }
}

#[component(inline_props)]
pub fn GalleryCategory(category: &'static GalleryCategory) -> View {
    let images = category.images.as_slice();
    let show = create_signal(false);
    let current_index = create_signal(0usize);

    let on_open: Rc<dyn Fn(usize)> = Rc::new(move |idx: usize| {
        current_index.set(idx);
        show.set(true);
    });

    view! {
        section(class="w-full flex flex-col gap-4") {
            h2(class="text-xl lxgw") { (category.name.clone()) }
            div(class="gallery-grid") {
                (images.iter().enumerate().map(|(idx, image)| {
                    let on_open = Rc::clone(&on_open);
                    view! {
                        GalleryCard(image=image, index=idx, on_open=on_open)
                    }
                }).collect::<Vec<_>>())
            }
            (move || {
                if show.get() {
                    view! {
                        GalleryLightbox(
                            images=images,
                            current_index=current_index,
                            show=show,
                        )
                    }
                } else {
                    view! {}
                }
            })
        }
    }
}

#[component(inline_props)]
pub fn GalleryCard(
    image: &'static GalleryImage,
    index: usize,
    on_open: Rc<dyn Fn(usize)>,
) -> View {
    let aspect_style = if image.width > 0 && image.height > 0 {
        format!("aspect-ratio: {}/{}", image.width, image.height)
    } else {
        "aspect-ratio: 4/3".to_string()
    };

    view! {
        div(
            class="gallery-card",
            style=aspect_style,
            on:click=move |_| on_open(index)
        ) {
            img(
                class="gallery-thumb",
                src=image.src.clone(),
                alt=image.title.clone().unwrap_or_default(),
                loading="lazy",
            )
            div(class="gallery-overlay") {
                span(class="gallery-title") { (image.title.clone().unwrap_or_default()) }
            }
        }
    }
}

#[component(inline_props)]
pub fn GalleryLightbox(
    images: &'static [GalleryImage],
    current_index: Signal<usize>,
    show: Signal<bool>,
) -> View {
    let close = move |_| show.set(false);

    let prev = move |_| {
        current_index.update(|i| {
            if *i == 0 {
                images.len() - 1
            } else {
                *i - 1
            }
        });
    };

    let next = move |_| {
        current_index.update(|i| {
            if *i + 1 >= images.len() {
                0
            } else {
                *i + 1
            }
        });
    };

    view! {
        div(class="gallery-lightbox") {
            button(class="gallery-lightbox-close", on:click=close) { "×" }
            button(class="gallery-lightbox-prev", on:click=prev) { "‹" }
            button(class="gallery-lightbox-next", on:click=next) { "›" }

            div(class="gallery-lightbox-content") {
                (move || {
                    let image = &images[current_index.get()];
                    view! {
                        img(
                            class="gallery-lightbox-img",
                            src=image.src.clone(),
                            alt=image.title.clone().unwrap_or_default(),
                        )
                        (image.title.clone().map(|title| {
                            view! { p(class="gallery-lightbox-title") { (title) } }
                        }))
                        (image.description.clone().map(|desc| {
                            view! { p(class="gallery-lightbox-desc") { (desc) } }
                        }))
                    }
                })
            }
        }
    }
}
