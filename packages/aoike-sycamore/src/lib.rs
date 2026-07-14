#[cfg(feature = "build")]
pub mod build;

pub mod docsgen;

use aoike::{GalleryCategory, GalleryImage, GalleryTimelineItem, PostData};
use wasm_bindgen::JsCast;
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

                    main(class=move || {
                        let is_gallery = matches!(route.get_clone(), AppRoutes::Gallery);
                        format!(
                            "w-full m-x-auto flex flex-col items-center p-8 gap-4 {}",
                            if is_gallery { "gallery-main" } else { "max-w-[120ch]" }
                        )
                    }) {
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

fn format_date(date: Option<aoike::time::Date>) -> String {
    match date {
        Some(d) => format!("{}年{}月{}日", d.year(), u8::from(d.month()), d.day()),
        None => "未知日期".to_string(),
    }
}

fn category_image_count(category: &'static GalleryCategory) -> usize {
    category.loose_images.len()
        + category.groups.iter().map(|g| g.images.len()).sum::<usize>()
}

#[component(inline_props)]
pub fn GalleryPage(categories: &'static [GalleryCategory]) -> View {
    let selected = create_signal(0usize);

    view! {
        div(class="gallery-page") {
            h1(class="gallery-page-title") { "Gallery" }
            (if categories.is_empty() {
                view! {
                    p(class="gallery-empty") { "还没有图片哦~" }
                }
            } else {
                view! {
                    div(class="gallery-tabs") {
                        (categories.iter().enumerate().map(|(idx, category)| {
                            view! {
                                button(
                                    class=move || format!("gallery-tab {}", if selected.get() == idx { "active" } else { "" }),
                                    on:click=move |_| selected.set(idx)
                                ) {
                                    (category.name.clone())
                                    span(class="gallery-tab-count") { (format!("{}", category_image_count(category))) }
                                }
                            }
                        }).collect::<Vec<_>>())
                    }
                    (move || {
                        let idx = selected.get();
                        if let Some(category) = categories.get(idx) {
                            view! {
                                GalleryCategoryTimeline(category=category)
                            }
                        } else {
                            view! {}
                        }
                    })
                }
            })
        }
    }
}

#[component(inline_props)]
pub fn GalleryCategoryTimeline(category: &'static GalleryCategory) -> View {
    let show = create_signal(false);
    let current_index = create_signal(0usize);

    // Pre-compute a static render description for each timeline date group.
    // Each entry contains the date, flat indices for loose images, and for each folder group
    // the group index plus the flat indices of its images.
    struct RenderGroup {
        group_idx: usize,
        indices: &'static [usize],
    }

    struct RenderDateGroup {
        date: Option<aoike::time::Date>,
        loose_indices: &'static [usize],
        groups: &'static [RenderGroup],
    }

    let mut all_images: Vec<&'static GalleryImage> = Vec::new();
    let mut render_items: Vec<RenderDateGroup> = Vec::new();

    for item in &category.timeline {
        match item {
            GalleryTimelineItem::DateGroup {
                date,
                loose_image_indices,
                folder_group_indices,
            } => {
                let loose_start = all_images.len();
                for &idx in loose_image_indices {
                    all_images.push(&category.loose_images[idx]);
                }
                let loose_indices: &'static [usize] =
                    Box::leak((loose_start..all_images.len()).collect::<Vec<_>>().into_boxed_slice());

                let mut groups = Vec::new();
                for &group_idx in folder_group_indices {
                    let group_start = all_images.len();
                    for img in &category.groups[group_idx].images {
                        all_images.push(img);
                    }
                    let indices: &'static [usize] =
                        Box::leak((group_start..all_images.len()).collect::<Vec<_>>().into_boxed_slice());
                    groups.push(RenderGroup { group_idx, indices });
                }
                let groups: &'static [RenderGroup] = Box::leak(groups.into_boxed_slice());

                render_items.push(RenderDateGroup {
                    date: *date,
                    loose_indices,
                    groups,
                });
            }
        }
    }

    let all_images: &'static [&'static GalleryImage] = Box::leak(all_images.into_boxed_slice());
    let render_items: &'static [RenderDateGroup] = Box::leak(render_items.into_boxed_slice());
    let total_items = render_items.len();

    // Lazy loading state: how many timeline date groups are currently rendered.
    let batch_size = 3usize;
    let visible_count = create_signal(batch_size.min(total_items));
    let sentinel_ref = create_node_ref();

    on_mount(move || {
        if let Some(sentinel) = sentinel_ref
            .try_get()
            .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
        {
            let visible_count = visible_count;
            let closure = wasm_bindgen::closure::Closure::wrap(Box::new(
                move |entries: js_sys::Array| {
                    if let Some(entry) = entries
                        .get(0)
                        .dyn_ref::<web_sys::IntersectionObserverEntry>()
                    {
                        if entry.is_intersecting() {
                            visible_count.update(|c| *c = (*c + batch_size).min(total_items));
                        }
                    }
                },
            )
                as Box<dyn FnMut(js_sys::Array)>);

            let observer = web_sys::IntersectionObserver::new_with_options(
                closure.as_ref().unchecked_ref(),
                &web_sys::IntersectionObserverInit::new(),
            );
            if let Ok(observer) = observer {
                observer.observe(&sentinel);
                closure.forget();
            }
        }
    });

    view! {
        section(class="gallery-category") {
            div(class="gallery-timeline") {
                (render_items.iter().take(visible_count.get()).map(|item| {
                    view! {
                        div(class="gallery-date-group") {
                            h2(class="gallery-date-heading lxgw") { (format_date(item.date)) }
                            div(class="gallery-date-content") {
                                (if !item.loose_indices.is_empty() {
                                    view! {
                                        div(class="gallery-masonry") {
                                            (item.loose_indices.iter().map(|&flat_idx| {
                                                view! {
                                                    GalleryCard(
                                                        image=all_images[flat_idx],
                                                        flat_index=flat_idx,
                                                        show=show,
                                                        current_index=current_index,
                                                    )
                                                }
                                            }).collect::<Vec<_>>())
                                        }
                                    }
                                } else {
                                    view! {}
                                })
                                (item.groups.iter().map(|group| {
                                    let gallery_group = &category.groups[group.group_idx];
                                    view! {
                                        div(class="gallery-folder-group") {
                                            div(class="gallery-folder-header") {
                                                h3(class="gallery-folder-heading lxgw") { (gallery_group.name.clone()) }
                                                (gallery_group.description_html.clone().map(|html| {
                                                    view! {
                                                        div(class="gallery-folder-desc markdown", dangerously_set_inner_html=html)
                                                    }
                                                }))
                                            }
                                            div(class="gallery-masonry") {
                                                (group.indices.iter().map(|&flat_idx| {
                                                    view! {
                                                        GalleryCard(
                                                            image=all_images[flat_idx],
                                                            flat_index=flat_idx,
                                                            show=show,
                                                            current_index=current_index,
                                                        )
                                                    }
                                                }).collect::<Vec<_>>())
                                            }
                                        }
                                    }
                                }).collect::<Vec<_>>())
                            }
                        }
                    }
                }).collect::<Vec<_>>())
            }
            (move || {
                if visible_count.get() < total_items {
                    view! {
                        div(class="gallery-sentinel", r#ref=sentinel_ref) { "加载更多…" }
                    }
                } else {
                    view! {}
                }
            })
            (move || {
                if show.get() {
                    view! {
                        GalleryLightbox(
                            images=all_images,
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
    flat_index: usize,
    show: Signal<bool>,
    current_index: Signal<usize>,
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
            on:click=move |_| {
                current_index.set(flat_index);
                show.set(true);
            }
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
    images: &'static [&'static GalleryImage],
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
                    let image = images[current_index.get()];
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
