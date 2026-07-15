#[cfg(feature = "build")]
pub mod build;

pub mod docsgen;

use aoike::{GalleryCategory, GalleryImage, GalleryTimelineItem, PostData};
use sycamore::prelude::*;
use sycamore_router::{HistoryIntegration, Route, Router, navigate, navigate_replace};
use wasm_bindgen::{JsCast, closure::Closure};

pub mod components {
    pub mod comment_overlay;
    pub mod giscus;
    pub mod waline;

    use sycamore::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum CommentSystem {
        Giscus(giscus::GiscusOptions),
        Waline(waline::WalineOptions),
    }

    pub fn render_comment_system(system: &CommentSystem, path: Option<String>) -> View {
        match system {
            CommentSystem::Giscus(options) => {
                let options = if let Some(path) = path {
                    options
                        .clone()
                        .with_mapping(giscus::Mapping::Specific(path))
                } else {
                    options.clone()
                };
                view! { giscus::Giscus(options=options) }
            }
            CommentSystem::Waline(options) => {
                let options = if let Some(path) = path {
                    options.clone().with_path(path)
                } else {
                    options.clone()
                };
                view! { waline::Waline(options=options) }
            }
        }
    }
}

use crate::{components::CommentSystem, layout::base::Header};

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
    #[to("/gallery/<slug>")]
    GalleryAlbum { slug: String },
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
    pub comment_system: Option<CommentSystem>,
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
                        let is_gallery = matches!(
                            route.get_clone(),
                            AppRoutes::Gallery | AppRoutes::GalleryAlbum { .. }
                        );
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
                                GalleryPage(categories=gallery, slug="".to_string())
                            },
                            AppRoutes::GalleryAlbum { slug } => view! {
                                GalleryPage(categories=gallery, slug=slug)
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

        (config.comment_system.clone().map(|system| {
            view! { components::comment_overlay::CommentOverlay(system=system, path="".to_string()) }
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

        (config.comment_system.clone().map(|system| {
            view! { components::comment_overlay::CommentOverlay(system=system, path="".to_string()) }
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
        + category
            .groups
            .iter()
            .map(|g| g.images.len())
        .sum::<usize>()
}

fn gallery_album_path(slug: &str) -> String {
    format!("/gallery/{}", slug)
}

#[component(inline_props)]
pub fn GalleryPage(categories: &'static [GalleryCategory], slug: String) -> View {
    let initial_idx = if slug.is_empty() {
        0
    } else {
        categories.iter().position(|c| c.slug == slug).unwrap_or(0)
    };
    let selected = create_signal(initial_idx);
    let config = use_context::<ConfigContext>();

    if categories.is_empty() {
        return view! {
            div(class="gallery-page") {
                h1(class="gallery-page-title") { "Gallery" }
                p(class="gallery-empty") { "还没有图片哦~" }
            }
        };
    }

    if slug.is_empty() {
        let canonical_path = gallery_album_path(&categories[initial_idx].slug);
        on_mount(move || navigate_replace(&canonical_path));
    }

    view! {
        div(class="gallery-page") {
            h1(class="gallery-page-title") { "Gallery" }
            div(class="gallery-tabs") {
                (categories.iter().enumerate().map(|(idx, category)| {
                    let category_slug = category.slug.clone();
                    view! {
                        button(
                            class=move || format!("gallery-tab {}", if selected.get() == idx { "active" } else { "" }),
                            on:click=move |_| {
                                selected.set(idx);
                                navigate(&gallery_album_path(&category_slug));
                            }
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
            (config.comment_system.clone().map(|system| {
                view! {
                    components::comment_overlay::CommentOverlay(
                        system=system,
                        path=gallery_album_path(&categories[initial_idx].slug),
                    )
                }
            }))
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
                let loose_indices: &'static [usize] = Box::leak(
                    (loose_start..all_images.len())
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                );

                let mut groups = Vec::new();
                for &group_idx in folder_group_indices {
                    let group_start = all_images.len();
                    for img in &category.groups[group_idx].images {
                        all_images.push(img);
                    }
                    let indices: &'static [usize] = Box::leak(
                        (group_start..all_images.len())
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                    );
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

    // Compute year groups for the year-filter tabs.
    #[derive(Clone)]
    struct YearGroup {
        year: i32,
        count: usize,
        item_indices: &'static [usize],
    }

    let mut year_counts = std::collections::BTreeMap::<i32, usize>::new();
    let mut year_item_indices = std::collections::BTreeMap::<i32, Vec<usize>>::new();

    for (idx, item) in render_items.iter().enumerate() {
        let count =
            item.loose_indices.len() + item.groups.iter().map(|g| g.indices.len()).sum::<usize>();
        if let Some(date) = item.date {
            let year = date.year();
            *year_counts.entry(year).or_insert(0) += count;
            year_item_indices.entry(year).or_default().push(idx);
        }
    }

    let year_groups: Vec<YearGroup> = year_item_indices
        .into_iter()
        .map(|(year, indices)| YearGroup {
            year,
            count: year_counts[&year],
            item_indices: Box::leak(indices.into_boxed_slice()),
        })
        .collect();
    let year_groups: &'static [YearGroup] = Box::leak(year_groups.into_boxed_slice());
    // Default to the most recent year that has images.
    let selected_year = create_signal(year_groups.last().map(|yg| yg.year));

    // When navigating the lightbox, scroll the current card into view.
    create_effect(move || {
        if !show.get() {
            return;
        }
        let flat_idx = current_index.get();
        let id = format!("gallery-img-{}", flat_idx);
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(element) = document.get_element_by_id(&id) {
                    let options = web_sys::ScrollIntoViewOptions::new();
                    options.set_block(web_sys::ScrollLogicalPosition::Center);
                    element.scroll_into_view_with_scroll_into_view_options(&options);
                }
            }
        }
    });

    view! {
        section(class="gallery-category") {
            (category.description_html.clone().map(|html| {
                view! {
                    div(class="gallery-category-desc markdown", dangerously_set_inner_html=html)
                }
            }))
            div(class="gallery-tabs gallery-year-tabs") {
                (year_groups.iter().map(|yg| {
                    let year = yg.year;
                    view! {
                        button(
                            class=move || format!("gallery-tab {}", if selected_year.get() == Some(year) { "active" } else { "" }),
                            on:click=move |_| selected_year.set(Some(year))
                        ) {
                            (format!("{}", year))
                            span(class="gallery-tab-count") { (format!("{}", yg.count)) }
                        }
                    }
                }).collect::<Vec<_>>())
            }
            div(class="gallery-timeline") {
                (move || {
                    let items: Vec<&RenderDateGroup> = selected_year
                        .get()
                        .and_then(|year| year_groups.iter().find(|yg| yg.year == year))
                        .map(|yg| yg.item_indices.iter().map(|&idx| &render_items[idx]).collect())
                        .unwrap_or_default();
                    items.into_iter().map(|item| {
                        view! {
                            div(class="gallery-date-group") {
                                h2(class="gallery-date-heading lxgw") { (format_date(item.date)) }
                                div(class="gallery-date-content") {
                                    (if !item.loose_indices.is_empty() {
                                        view! {
                                            GalleryRows(
                                                images=all_images,
                                                flat_indices=item.loose_indices,
                                                show=show,
                                                current_index=current_index,
                                            )
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
                                                GalleryRows(
                                                    images=all_images,
                                                    flat_indices=group.indices,
                                                    show=show,
                                                    current_index=current_index,
                                                )
                                            }
                                        }
                                    }).collect::<Vec<_>>())
                                }
                            }
                        }
                    }).collect::<Vec<_>>()
                })
            }
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

fn is_red_label(image: &GalleryImage) -> bool {
    image
        .label
        .as_deref()
        .map(|l| l.eq_ignore_ascii_case("Red"))
        .unwrap_or(false)
}

fn image_rating(image: &GalleryImage) -> u8 {
    image.rating.unwrap_or(3).clamp(3, 5)
}

fn aspect_ratio(image: &GalleryImage) -> f64 {
    if image.width > 0 && image.height > 0 {
        (image.width as f64 / image.height as f64).clamp(0.25, 4.0)
    } else {
        4.0 / 3.0
    }
}

fn layout_aspect_ratio(image: &GalleryImage) -> f64 {
    let aspect = aspect_ratio(image);
    if aspect < 0.85 {
        (aspect + (0.85 - aspect) * 0.32).min(aspect / 0.86)
    } else if aspect > 1.9 {
        (aspect - (aspect - 1.9) * 0.28).max(aspect * 0.88)
    } else {
        aspect
    }
}

fn portrait_pressure(image: &GalleryImage) -> f64 {
    ((0.9 - aspect_ratio(image)) / 0.5).clamp(0.0, 1.0)
}

fn row_portrait_pressure(images: &[&GalleryImage]) -> f64 {
    if images.is_empty() {
        return 0.0;
    }

    let max_pressure = images
        .iter()
        .map(|image| portrait_pressure(image))
        .fold(0.0_f64, f64::max);
    let average_pressure = images
        .iter()
        .map(|image| portrait_pressure(image))
        .sum::<f64>()
        / images.len() as f64;
    let count_attenuation = if images.len() <= 2 {
        1.0
    } else {
        (1.0 - 0.12 * (images.len() - 2) as f64).max(0.64)
    };

    (max_pressure * 0.75 + average_pressure * 0.25) * count_attenuation
}

const DEFAULT_GALLERY_WIDTH: f64 = 960.0;
const DEFAULT_VIEWPORT_HEIGHT: f64 = 900.0;
const MAX_STANDARD_ROW_ITEMS: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GalleryRowKind {
    Standard,
    Feature,
}

#[derive(Clone, Debug, PartialEq)]
struct GalleryLayoutItem {
    flat_index: usize,
    width: f64,
    height: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct GalleryLayoutRow {
    kind: GalleryRowKind,
    items: Vec<GalleryLayoutItem>,
}

struct GalleryResizeObserver {
    observer: web_sys::ResizeObserver,
    _closure: Closure<dyn FnMut(js_sys::Array)>,
}

impl Drop for GalleryResizeObserver {
    fn drop(&mut self) {
        self.observer.disconnect();
    }
}

fn gallery_gap(container_width: f64) -> f64 {
    if container_width < 640.0 { 8.0 } else { 12.0 }
}

fn standard_base_height(container_width: f64) -> f64 {
    if container_width < 640.0 {
        (container_width / 2.35).clamp(145.0, 175.0)
    } else {
        (container_width / 5.2).clamp(185.0, 260.0)
    }
}

fn feature_max_height(container_width: f64, viewport_height: f64) -> f64 {
    if container_width < 640.0 {
        (viewport_height * 0.48).min(420.0)
    } else if container_width < 1000.0 {
        (viewport_height * 0.56).min(520.0)
    } else {
        (viewport_height * 0.62).min(620.0)
    }
}

fn single_standard_max_height(container_width: f64, viewport_height: f64) -> f64 {
    if container_width < 640.0 {
        (viewport_height * 0.46).min(360.0)
    } else if container_width < 1000.0 {
        (viewport_height * 0.50).min(420.0)
    } else {
        (viewport_height * 0.54).min(480.0)
    }
}

fn segment_has_rating_contrast(images: &[&GalleryImage]) -> bool {
    let min = images.iter().map(|image| image_rating(image)).min();
    let max = images.iter().map(|image| image_rating(image)).max();
    min != max
}

fn row_target_height(images: &[&GalleryImage], base_height: f64, has_contrast: bool) -> f64 {
    if !has_contrast || images.is_empty() {
        return base_height;
    }

    let (weighted_priority, aspect_sum) = images.iter().fold((0.0, 0.0), |acc, image| {
        let aspect = aspect_ratio(image);
        let priority = image_rating(image) as f64 - 4.0;
        (acc.0 + priority * aspect, acc.1 + aspect)
    });
    let average_priority = if aspect_sum > 0.0 {
        weighted_priority / aspect_sum
    } else {
        0.0
    };

    base_height * (1.0 + average_priority.clamp(-1.0, 1.0) * 0.08)
}

fn make_layout_item(flat_index: usize, image: &GalleryImage, height: f64) -> GalleryLayoutItem {
    GalleryLayoutItem {
        flat_index,
        width: layout_aspect_ratio(image) * height,
        height,
    }
}

fn standard_row_candidate(
    images: &[&GalleryImage],
    flat_indices: &[usize],
    container_width: f64,
    gap: f64,
    base_height: f64,
    max_height: f64,
    single_max_height: f64,
    has_contrast: bool,
    is_last: bool,
    segment_len: usize,
) -> Option<(GalleryLayoutRow, f64)> {
    let count = images.len();
    let available_width = container_width - gap * count.saturating_sub(1) as f64;
    let aspect_sum: f64 = images.iter().map(|image| layout_aspect_ratio(image)).sum();
    if count == 0 || available_width <= 0.0 || aspect_sum <= 0.0 {
        return None;
    }

    let target_height = row_target_height(images, base_height, has_contrast);
    let portrait_pressure = row_portrait_pressure(images);
    let target_height = if count == 1 {
        target_height
    } else {
        target_height * (1.0 + portrait_pressure * 0.45)
    };
    let row_max_height = max_height * (1.0 + portrait_pressure * 0.55);
    let ideal_height = available_width / aspect_sum;
    let normal_min_height = if container_width < 640.0 {
        92.0
    } else {
        base_height * 0.68
    };
    let portrait_floor = target_height * (0.90 + portrait_pressure * 0.12);
    let portrait_floor_influence = (portrait_pressure / 0.35).clamp(0.0, 1.0);
    let min_height = normal_min_height
        + (portrait_floor.max(normal_min_height) - normal_min_height)
            * portrait_floor_influence;

    if count > 1 && ideal_height < min_height {
        return None;
    }

    let height = if count == 1 {
        let aspect = aspect_sum;
        let is_portrait = aspect < 0.85;
        let base_scale = if aspect < 1.15 { 1.45 } else { 1.30 };
        let height_scale = base_scale + portrait_pressure * 0.70;
        let single_max_height = single_max_height * (1.0 + portrait_pressure * 0.15);
        let width_fraction = if container_width < 640.0 {
            if is_portrait { 0.72 } else { 1.0 }
        } else if is_portrait {
            0.46
        } else {
            0.84
        };
        let width_limited_height = container_width * width_fraction / aspect;
        (target_height * height_scale)
            .min(single_max_height)
            .min(width_limited_height)
    } else if is_last && ideal_height > target_height * 1.08 {
        target_height.min(row_max_height)
    } else {
        ideal_height.min(row_max_height)
    };

    let items: Vec<GalleryLayoutItem> = flat_indices
        .iter()
        .zip(images)
        .map(|(&flat_index, image)| make_layout_item(flat_index, image, height))
        .collect();
    let used_width =
        items.iter().map(|item| item.width).sum::<f64>() + gap * count.saturating_sub(1) as f64;
    if used_width > container_width + 0.5 {
        return None;
    }

    let height_error = (height - target_height) / base_height;
    let slack_ratio = ((container_width - used_width) / container_width).max(0.0);
    let mut cost = height_error * height_error * 4.0
        + slack_ratio * slack_ratio * if is_last { 0.45 } else { 2.5 };

    if count == 1 && segment_len > 1 {
        cost += 7.0;
    }

    for (item, image) in items.iter().zip(images) {
        if !has_contrast {
            continue;
        }
        let share = item.width / container_width;
        match image_rating(image) {
            5 => {
                if share < 0.18 {
                    cost += (0.18 - share).powi(2) * 16.0;
                }
                if count > 4 {
                    cost += 0.18 * (count - 4) as f64;
                }
            }
            3 if share > 0.48 => {
                cost += (share - 0.48).powi(2) * 12.0;
            }
            _ => {}
        }
    }

    Some((
        GalleryLayoutRow {
            kind: GalleryRowKind::Standard,
            items,
        },
        cost,
    ))
}

fn layout_standard_segment(
    all_images: &[&GalleryImage],
    flat_indices: &[usize],
    container_width: f64,
    viewport_height: f64,
    gap: f64,
) -> Vec<GalleryLayoutRow> {
    let segment_images: Vec<&GalleryImage> = flat_indices
        .iter()
        .map(|&flat_index| all_images[flat_index])
        .collect();
    let len = segment_images.len();
    if len == 0 {
        return Vec::new();
    }

    let base_height = standard_base_height(container_width);
    let max_height = if container_width < 640.0 {
        220.0
    } else {
        (base_height * 1.32).min(330.0)
    };
    let single_max_height = single_standard_max_height(container_width, viewport_height);
    let has_contrast = segment_has_rating_contrast(&segment_images);
    let max_items = if container_width < 640.0 {
        4
    } else {
        MAX_STANDARD_ROW_ITEMS
    };

    let mut costs = vec![f64::INFINITY; len + 1];
    let mut previous: Vec<Option<(usize, GalleryLayoutRow)>> = vec![None; len + 1];
    costs[0] = 0.0;

    for start in 0..len {
        if !costs[start].is_finite() {
            continue;
        }
        for end in (start + 1)..=(start + max_items).min(len) {
            let candidate = standard_row_candidate(
                &segment_images[start..end],
                &flat_indices[start..end],
                container_width,
                gap,
                base_height,
                max_height,
                single_max_height,
                has_contrast,
                end == len,
                len,
            );
            let Some((row, row_cost)) = candidate else {
                continue;
            };
            let total_cost = costs[start] + row_cost;
            if total_cost < costs[end] {
                costs[end] = total_cost;
                previous[end] = Some((start, row));
            }
        }
    }

    let mut cursor = len;
    let mut rows = Vec::new();
    while cursor > 0 {
        let Some((start, row)) = previous[cursor].take() else {
            let image = segment_images[cursor - 1];
            let aspect = layout_aspect_ratio(image);
            let is_portrait = aspect < 0.85;
            let width_fraction = if container_width < 640.0 {
                if is_portrait { 0.72 } else { 1.0 }
            } else if is_portrait {
                0.46
            } else {
                0.84
            };
            let pressure = portrait_pressure(image);
            let base_scale = if aspect < 1.15 { 1.45 } else { 1.30 };
            let height_scale = base_scale + pressure * 0.70;
            let single_max_height = single_max_height * (1.0 + pressure * 0.15);
            let height = (base_height * height_scale)
                .min(single_max_height)
                .min(container_width * width_fraction / aspect);
            rows.push(GalleryLayoutRow {
                kind: GalleryRowKind::Standard,
                items: vec![make_layout_item(flat_indices[cursor - 1], image, height)],
            });
            cursor -= 1;
            continue;
        };
        rows.push(row);
        cursor = start;
    }
    rows.reverse();
    rows
}

fn feature_row_candidate(
    images: &[&GalleryImage],
    flat_indices: &[usize],
    container_width: f64,
    gap: f64,
    max_height: f64,
) -> (GalleryLayoutRow, f64) {
    let count = images.len();
    let available_width = container_width - gap * count.saturating_sub(1) as f64;
    let aspect_sum: f64 = images.iter().map(|image| layout_aspect_ratio(image)).sum();
    let ideal_height = available_width / aspect_sum;
    let portrait_pressure = row_portrait_pressure(images);
    let row_max_height = max_height * (1.0 + portrait_pressure * 0.18);
    let height = ideal_height.min(row_max_height);
    let items: Vec<GalleryLayoutItem> = flat_indices
        .iter()
        .zip(images)
        .map(|(&flat_index, image)| make_layout_item(flat_index, image, height))
        .collect();
    let used_width =
        items.iter().map(|item| item.width).sum::<f64>() + gap * count.saturating_sub(1) as f64;
    let slack_ratio = ((container_width - used_width) / container_width).max(0.0);
    let height_error = (height - row_max_height) / row_max_height.max(1.0);
    let cost = slack_ratio * slack_ratio * 2.0 + height_error * height_error * 0.35;

    (
        GalleryLayoutRow {
            kind: GalleryRowKind::Feature,
            items,
        },
        cost,
    )
}

fn layout_feature_segment(
    all_images: &[&GalleryImage],
    flat_indices: &[usize],
    container_width: f64,
    viewport_height: f64,
    gap: f64,
) -> Vec<GalleryLayoutRow> {
    let segment_images: Vec<&GalleryImage> = flat_indices
        .iter()
        .map(|&flat_index| all_images[flat_index])
        .collect();
    let len = segment_images.len();
    if len == 0 {
        return Vec::new();
    }

    let max_height = feature_max_height(container_width, viewport_height);
    let max_items = if container_width < 640.0 { 1 } else { 2 };
    let mut costs = vec![f64::INFINITY; len + 1];
    let mut previous: Vec<Option<(usize, GalleryLayoutRow)>> = vec![None; len + 1];
    costs[0] = 0.0;

    for start in 0..len {
        for end in (start + 1)..=(start + max_items).min(len) {
            let (row, mut row_cost) = feature_row_candidate(
                &segment_images[start..end],
                &flat_indices[start..end],
                container_width,
                gap,
                max_height,
            );
            if end - start == 1 && len > 1 {
                row_cost += 0.08;
            }
            let total_cost = costs[start] + row_cost;
            if total_cost < costs[end] {
                costs[end] = total_cost;
                previous[end] = Some((start, row));
            }
        }
    }

    let mut cursor = len;
    let mut rows = Vec::new();
    while cursor > 0 {
        let (start, row) = previous[cursor]
            .take()
            .expect("feature layout always has a single-image fallback");
        rows.push(row);
        cursor = start;
    }
    rows.reverse();
    rows
}

fn compute_gallery_layouts(
    images: &[&GalleryImage],
    flat_indices: &[usize],
    container_width: f64,
    viewport_height: f64,
) -> Vec<GalleryLayoutRow> {
    if flat_indices.is_empty() || container_width <= 0.0 {
        return Vec::new();
    }

    let gap = gallery_gap(container_width);
    let mut rows = Vec::new();
    let (feature_indices, standard_indices): (Vec<usize>, Vec<usize>) = flat_indices
        .iter()
        .copied()
        .partition(|&flat_index| is_red_label(images[flat_index]));

    if !feature_indices.is_empty() {
        rows.extend(layout_feature_segment(
            images,
            &feature_indices,
            container_width,
            viewport_height,
            gap,
        ));
    }
    if !standard_indices.is_empty() {
        rows.extend(layout_standard_segment(
            images,
            &standard_indices,
            container_width,
            viewport_height,
            gap,
        ));
    }
    rows
}

#[component(inline_props)]
fn GalleryRows(
    images: &'static [&'static GalleryImage],
    flat_indices: &'static [usize],
    show: Signal<bool>,
    current_index: Signal<usize>,
) -> View {
    use std::{cell::RefCell, rc::Rc};

    let container_ref = create_node_ref();
    let container_width = create_signal(DEFAULT_GALLERY_WIDTH);
    let viewport_height = create_signal(DEFAULT_VIEWPORT_HEIGHT);
    let resize_observer = Rc::new(RefCell::new(None::<GalleryResizeObserver>));

    on_mount({
        let resize_observer = resize_observer.clone();
        move || {
            let Some(element) = container_ref
                .try_get()
                .and_then(|node| node.dyn_into::<web_sys::Element>().ok())
            else {
                return;
            };

            let set_dimensions = move |width: f64| {
                if (container_width.get_untracked() - width).abs() > 4.0 {
                    container_width.set(width.max(1.0));
                }
                if let Some(height) = web_sys::window()
                    .and_then(|window| window.inner_height().ok())
                    .and_then(|height| height.as_f64())
                    .filter(|height| (viewport_height.get_untracked() - height).abs() > 1.0)
                {
                    viewport_height.set(height);
                }
            };
            set_dimensions(element.get_bounding_client_rect().width());

            let closure = Closure::wrap(Box::new(move |entries: js_sys::Array| {
                let Some(entry) = entries
                    .get(0)
                    .dyn_into::<web_sys::ResizeObserverEntry>()
                    .ok()
                else {
                    return;
                };
                set_dimensions(entry.content_rect().width());
            }) as Box<dyn FnMut(js_sys::Array)>);

            if let Ok(observer) = web_sys::ResizeObserver::new(closure.as_ref().unchecked_ref()) {
                observer.observe(&element);
                resize_observer.borrow_mut().replace(GalleryResizeObserver {
                    observer,
                    _closure: closure,
                });
            }
        }
    });

    on_cleanup({
        let resize_observer = resize_observer.clone();
        move || {
            resize_observer.borrow_mut().take();
        }
    });

    view! {
        div(
            class="gallery-layout",
            r#ref=container_ref,
            style=move || format!("--gallery-row-gap: {}px", gallery_gap(container_width.get()))
        ) {
            (move || {
                compute_gallery_layouts(
                    images,
                    flat_indices,
                    container_width.get(),
                    viewport_height.get(),
                )
                .into_iter()
                .map(|row| {
                    let row_kind = row.kind;
                    let row_items = row.items;
                    let row_class = match row_kind {
                        GalleryRowKind::Standard => "gallery-row gallery-row-standard",
                        GalleryRowKind::Feature => "gallery-row gallery-row-feature",
                    };
                    view! {
                        div(class=row_class) {
                            (row_items.iter().map(|item| {
                                view! {
                                    GalleryCard(
                                        image=images[item.flat_index],
                                        flat_index=item.flat_index,
                                        show=show,
                                        current_index=current_index,
                                        width_px=item.width,
                                        height_px=item.height,
                                        is_feature=row_kind == GalleryRowKind::Feature,
                                    )
                                }
                            }).collect::<Vec<_>>())
                        }
                    }
                })
                .collect::<Vec<_>>()
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
    width_px: f64,
    height_px: f64,
    is_feature: bool,
) -> View {
    let rating_class = match image_rating(image) {
        4 => "gallery-card-rating-4",
        5 => "gallery-card-rating-5",
        _ => "gallery-card-rating-3",
    };
    let card_class = format!(
        "gallery-card {} {}",
        rating_class,
        if is_feature {
            "gallery-card-feature"
        } else {
            ""
        },
    );
    let style = format!("width: {}px; height: {}px", width_px, height_px);

    view! {
        div(
            class=card_class,
            id=format!("gallery-img-{}", flat_index),
            style=style,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn img(rating: Option<u8>, label: Option<&str>, width: u32, height: u32) -> GalleryImage {
        GalleryImage {
            src: String::new(),
            thumb_src: None,
            width,
            height,
            title: None,
            description: None,
            created: None,
            rating,
            label: label.map(|s| s.to_string()),
        }
    }

    #[test]
    fn row_target_is_neutral_when_all_ratings_match() {
        let a = img(Some(3), None, 400, 300);
        let b = img(Some(3), None, 400, 300);
        let c = img(None, None, 400, 300);
        let images = vec![&a, &b, &c];
        assert!(!segment_has_rating_contrast(&images));
        assert_eq!(row_target_height(&images, 200.0, false), 200.0);
    }

    #[test]
    fn gallery_album_path_is_shared_by_routes_and_comments() {
        assert_eq!(gallery_album_path("earth-online"), "/gallery/earth-online");
    }

    #[test]
    fn fixed_rating_semantics_do_not_min_max_normalize() {
        let a = img(Some(3), None, 400, 300);
        let b = img(Some(4), None, 400, 300);
        let c = img(Some(5), None, 400, 300);
        assert_eq!(image_rating(&a) as i8 - 4, -1);
        assert_eq!(image_rating(&b) as i8 - 4, 0);
        assert_eq!(image_rating(&c) as i8 - 4, 1);
    }

    #[test]
    fn red_images_are_moved_to_the_top_feature_region() {
        let a = img(Some(3), None, 400, 300);
        let b = img(Some(5), Some("Red"), 400, 300);
        let c = img(Some(4), None, 400, 300);
        let images = vec![&a, &b, &c];
        let rows = compute_gallery_layouts(&images, &[0, 1, 2], 1200.0, 900.0);

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].kind, GalleryRowKind::Feature);
        assert_eq!(rows[0].items.len(), 1);
        assert_eq!(rows[0].items[0].flat_index, 1);
        let ordinary_order: Vec<usize> = rows[1].items.iter().map(|item| item.flat_index).collect();
        assert_eq!(ordinary_order, vec![0, 2]);
    }

    #[test]
    fn ordinary_single_image_uses_the_larger_single_item_target() {
        let image = img(Some(5), None, 1600, 900);
        let images = vec![&image];
        let rows = compute_gallery_layouts(&images, &[0], 1200.0, 900.0);
        let item = &rows[0].items[0];

        assert_eq!(rows[0].kind, GalleryRowKind::Standard);
        assert!(item.width <= 1200.0 * 0.84 + 0.5);
        assert!(item.height > 270.0);
        assert!(item.height <= 480.0);
        assert!((item.width / item.height - 1600.0 / 900.0).abs() < 1e-9);
    }

    #[test]
    fn single_portrait_is_taller_than_a_normal_row() {
        let image = img(Some(3), None, 600, 1600);
        let images = vec![&image];
        let rows = compute_gallery_layouts(&images, &[0], 760.0, 900.0);
        let item = &rows[0].items[0];

        assert!(item.height > standard_base_height(760.0) * 1.9);
        assert!(item.height <= single_standard_max_height(760.0, 900.0));
        assert!(item.width > 160.0);
    }

    #[test]
    fn extreme_aspect_ratios_are_only_cropped_slightly() {
        let portrait = img(None, None, 400, 1200);
        let panorama = img(None, None, 3000, 1000);
        let portrait_aspect = aspect_ratio(&portrait);
        let portrait_layout = layout_aspect_ratio(&portrait);
        let panorama_aspect = aspect_ratio(&panorama);
        let panorama_layout = layout_aspect_ratio(&panorama);

        assert!(portrait_layout > portrait_aspect);
        assert!(portrait_aspect / portrait_layout >= 0.86 - 1e-9);
        assert!(panorama_layout < panorama_aspect);
        assert!(panorama_layout / panorama_aspect >= 0.88 - 1e-9);
    }

    #[test]
    fn portrait_pressure_does_not_raise_landscape_only_rows() {
        let landscape_a = img(None, None, 1600, 900);
        let landscape_b = img(None, None, 1600, 900);
        let portrait = img(None, None, 600, 1200);
        let landscape_images = vec![&landscape_a, &landscape_b];
        let mixed_images = vec![&portrait, &landscape_b];

        assert_eq!(row_portrait_pressure(&landscape_images), 0.0);

        let (landscape_row, _) = standard_row_candidate(
            &landscape_images,
            &[0, 1],
            1000.0,
            12.0,
            200.0,
            260.0,
            420.0,
            false,
            false,
            2,
        )
        .unwrap();
        let (mixed_row, _) = standard_row_candidate(
            &mixed_images,
            &[0, 1],
            1000.0,
            12.0,
            200.0,
            260.0,
            420.0,
            false,
            false,
            2,
        )
        .unwrap();

        assert!(mixed_row.items[0].height > landscape_row.items[0].height);
    }

    #[test]
    fn portrait_pressure_weakens_in_crowded_rows() {
        let portrait = img(None, None, 400, 1200);
        let landscape_a = img(None, None, 1600, 900);
        let landscape_b = img(None, None, 1600, 900);
        let landscape_c = img(None, None, 1600, 900);
        let landscape_d = img(None, None, 1600, 900);

        let pair_pressure = row_portrait_pressure(&[&portrait, &landscape_a]);
        let crowded_pressure = row_portrait_pressure(&[
            &portrait,
            &landscape_a,
            &landscape_b,
            &landscape_c,
            &landscape_d,
        ]);

        assert!(crowded_pressure < pair_pressure);
        assert!(crowded_pressure > 0.0);
    }

    #[test]
    fn portrait_heavy_five_image_group_splits_instead_of_compressing() {
        let portrait_a = img(None, None, 1360, 2417);
        let portrait_b = img(None, None, 1360, 2417);
        let portrait_c = img(None, None, 1360, 2417);
        let landscape_a = img(None, None, 3440, 1360);
        let landscape_b = img(None, None, 3440, 1360);
        let images = vec![
            &portrait_a,
            &portrait_b,
            &portrait_c,
            &landscape_a,
            &landscape_b,
        ];

        let rows = compute_gallery_layouts(&images, &[0, 1, 2, 3, 4], 1740.0, 900.0);

        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|row| row.items.len() < 5));
        let tallest_portrait = rows
            .iter()
            .flat_map(|row| row.items.iter())
            .filter(|item| item.flat_index < 3)
            .map(|item| item.height)
            .fold(0.0_f64, f64::max);
        assert!(tallest_portrait > 300.0);
    }

    #[test]
    fn feature_height_is_capped_and_aspect_ratio_is_preserved() {
        let square = img(Some(5), Some("Red"), 800, 800);
        let images = vec![&square];
        let rows = compute_gallery_layouts(&images, &[0], 1400.0, 900.0);
        let item = &rows[0].items[0];

        assert_eq!(rows[0].kind, GalleryRowKind::Feature);
        assert!(item.height <= 558.0 + 0.5);
        assert!((item.width - item.height).abs() < 1e-9);
    }

    #[test]
    fn consecutive_feature_images_pair_on_desktop_and_stack_on_mobile() {
        let detail = img(Some(5), Some("Red"), 800, 800);
        let landscape = img(Some(5), Some("Red"), 1600, 900);
        let images = vec![&detail, &landscape];

        let desktop = compute_gallery_layouts(&images, &[0, 1], 1400.0, 900.0);
        assert_eq!(desktop.len(), 1);
        assert_eq!(desktop[0].kind, GalleryRowKind::Feature);
        assert_eq!(desktop[0].items.len(), 2);

        let mobile = compute_gallery_layouts(&images, &[0, 1], 360.0, 800.0);
        assert_eq!(mobile.len(), 2);
        assert!(mobile.iter().all(|row| row.items.len() == 1));
    }

    #[test]
    fn layout_preserves_image_order() {
        let a = img(Some(4), None, 1200, 800);
        let b = img(Some(5), None, 800, 1200);
        let c = img(Some(3), None, 1600, 900);
        let d = img(Some(5), None, 1000, 1000);
        let images = vec![&a, &b, &c, &d];
        let rows = compute_gallery_layouts(&images, &[0, 1, 2, 3], 1000.0, 800.0);
        let order: Vec<usize> = rows
            .iter()
            .flat_map(|row| row.items.iter().map(|item| item.flat_index))
            .collect();

        assert_eq!(order, vec![0, 1, 2, 3]);
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
            *i = if *i == 0 { images.len() - 1 } else { *i - 1 };
        });
    };

    let next = move |_| {
        current_index.update(|i| {
            *i = if *i + 1 >= images.len() { 0 } else { *i + 1 };
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
                        (if image.rating.is_some() || image.label.is_some() {
                            let rating_stars = image.rating.map(gallery_rating_stars).unwrap_or_default();
                            let rating_aria = image.rating
                                .map(|rating| format!("{} stars", rating))
                                .unwrap_or_default();
                            let rating_class = if image.rating.is_some() {
                                "gallery-lightbox-rating"
                            } else {
                                "gallery-lightbox-rating hidden"
                            };
                            let label = image.label.clone().unwrap_or_default();
                            let label_class = if image.label.is_some() {
                                format!("gallery-lightbox-label {}", gallery_label_class(&label))
                            } else {
                                "gallery-lightbox-label hidden".to_string()
                            };
                            view! {
                                div(class="gallery-lightbox-meta") {
                                    span(class=rating_class, aria-label=rating_aria) { (rating_stars.clone()) }
                                    span(class=label_class) {
                                        span(class="gallery-lightbox-label-swatch", aria-hidden="true")
                                        span { (label.clone()) }
                                    }
                                }
                            }
                        } else {
                            view! {}
                        })
                        (image.description.clone().map(|desc| {
                            view! { p(class="gallery-lightbox-desc") { (desc) } }
                        }))
                    }
                })
            }
        }
    }
}

fn gallery_rating_stars(rating: u8) -> String {
    let rating = rating.clamp(1, 5) as usize;
    format!("{}{}", "★".repeat(rating), "☆".repeat(5 - rating))
}

fn gallery_label_class(label: &str) -> &'static str {
    match label.to_ascii_lowercase().as_str() {
        "red" => "gallery-lightbox-label-red",
        "yellow" => "gallery-lightbox-label-yellow",
        "green" => "gallery-lightbox-label-green",
        "blue" => "gallery-lightbox-label-blue",
        "purple" => "gallery-lightbox-label-purple",
        _ => "gallery-lightbox-label-default",
    }
}

#[cfg(test)]
mod lightbox_metadata_tests {
    use super::*;

    #[test]
    fn formats_rating_as_five_star_state() {
        assert_eq!(gallery_rating_stars(3), "★★★☆☆");
        assert_eq!(gallery_rating_stars(5), "★★★★★");
    }

    #[test]
    fn maps_common_xmp_labels_to_swatch_classes() {
        assert_eq!(gallery_label_class("Red"), "gallery-lightbox-label-red");
        assert_eq!(
            gallery_label_class("purple"),
            "gallery-lightbox-label-purple"
        );
        assert_eq!(
            gallery_label_class("Custom"),
            "gallery-lightbox-label-default"
        );
    }
}
