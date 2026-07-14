use std::path::Path;

use anyhow::Context;
use exif::{Exif, In, Reader, Tag, Value};
use walkdir::WalkDir;

use crate::{time::Date, GalleryCategory, GalleryGroup, GalleryImage, GalleryTimelineItem};

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif"];

pub fn parse_gallery(dir: impl AsRef<Path>) -> Vec<GalleryCategory> {
    let dir = dir.as_ref();
    if !dir.exists() {
        return Vec::new();
    }

    let mut categories = Vec::new();

    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        if let Some(category) = parse_category(&path) {
            categories.push(category);
        }
    }

    categories.sort_by(|a, b| a.name.cmp(&b.name));
    categories
}

fn parse_category(path: &Path) -> Option<GalleryCategory> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unnamed")
        .to_string();
    let slug = slug::slugify(&name);
    let src_prefix = format!("/static/gallery/{}", name);

    let description_html = path
        .join("index.md")
        .exists()
        .then(|| parse_markdown_file(&path.join("index.md")))
        .flatten();

    let mut loose_images = Vec::new();
    let mut groups = Vec::new();

    for entry in std::fs::read_dir(path).unwrap().flatten() {
        let entry_path = entry.path();

        if entry_path.is_dir() {
            if let Some(group) = parse_group(&entry_path, &src_prefix) {
                if !group.images.is_empty() {
                    groups.push(group);
                }
            }
        } else if is_image(&entry_path) {
            if let Some(image) = parse_image(&entry_path, &src_prefix) {
                loose_images.push(image);
            }
        }
    }

    loose_images.sort_by(|a, b| b.created.cmp(&a.created));

    let timeline = build_timeline(&loose_images, &groups);

    Some(GalleryCategory {
        name,
        slug,
        description_html,
        loose_images,
        groups,
        timeline,
    })
}

fn parse_group(path: &Path, category_src_prefix: &str) -> Option<GalleryGroup> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unnamed")
        .to_string();
    let slug = slug::slugify(&name);
    let src_prefix = format!("{}/{}", category_src_prefix, name);

    let description_html = path
        .join("index.md")
        .exists()
        .then(|| parse_markdown_file(&path.join("index.md")))
        .flatten();

    let images: Vec<GalleryImage> = WalkDir::new(path)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file() && is_image(e.path()))
        .filter_map(|e| parse_image(e.path(), &src_prefix))
        .collect();

    // Preserve filesystem order within a group; don't sort by time.
    // (If the user wants a specific order, they can rename files.)
    Some(GalleryGroup {
        name,
        slug,
        description_html,
        images,
    })
}

fn parse_markdown_file(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let parser = pulldown_cmark::Parser::new(&content);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    Some(html)
}

fn build_timeline(
    loose_images: &[GalleryImage],
    groups: &[GalleryGroup],
) -> Vec<GalleryTimelineItem> {
    use std::collections::BTreeMap;

    #[derive(Default)]
    struct DateContent {
        loose_image_indices: Vec<usize>,
        folder_group_indices: Vec<usize>,
    }

    let mut by_date: BTreeMap<Option<Date>, DateContent> = BTreeMap::new();

    // Loose images grouped by date
    for (idx, image) in loose_images.iter().enumerate() {
        let date = image.created.map(|dt| dt.date());
        by_date.entry(date).or_default().loose_image_indices.push(idx);
    }

    // Sort loose images within each date by datetime descending
    for content in by_date.values_mut() {
        content.loose_image_indices.sort_by(|&a, &b| {
            let a_dt = loose_images[a].created;
            let b_dt = loose_images[b].created;
            b_dt.cmp(&a_dt)
        });
    }

    // Folder groups placed at their newest image date
    let mut group_entries: Vec<(Option<Date>, usize)> = Vec::new();
    for (group_idx, group) in groups.iter().enumerate() {
        let date = group
            .images
            .iter()
            .filter_map(|img| img.created)
            .max()
            .map(|dt| dt.date());
        group_entries.push((date, group_idx));
    }

    // Sort folder groups within the same date by their newest datetime descending
    group_entries.sort_by(|a, b| {
        let a_max = groups[a.1]
            .images
            .iter()
            .filter_map(|img| img.created)
            .max();
        let b_max = groups[b.1]
            .images
            .iter()
            .filter_map(|img| img.created)
            .max();
        b_max.cmp(&a_max)
    });

    for (date, group_idx) in group_entries {
        by_date.entry(date).or_default().folder_group_indices.push(group_idx);
    }

    // Convert to items, sorted descending by date; None (unknown) goes last.
    let mut items: Vec<(Option<Date>, GalleryTimelineItem)> = by_date
        .into_iter()
        .map(|(date, content)| {
            (
                date,
                GalleryTimelineItem::DateGroup {
                    date,
                    loose_image_indices: content.loose_image_indices,
                    folder_group_indices: content.folder_group_indices,
                },
            )
        })
        .collect();
    items.sort_by(|a, b| b.0.cmp(&a.0));

    items.into_iter().map(|(_, item)| item).collect()
}

fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| IMAGE_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn parse_image(path: &Path, src_prefix: &str) -> Option<GalleryImage> {
    if !is_image(path) {
        return None;
    }

    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image")
        .to_string();

    let src = format!("{}/{}", src_prefix, path.file_name()?.to_str()?);

    let exif = read_exif(path).ok();
    let (width, height) = exif
        .as_ref()
        .and_then(image_dimensions_from_exif)
        .or_else(|| imagesize::size(path).ok().map(|s| (s.width as u32, s.height as u32)))
        .unwrap_or((0, 0));

    let title = exif
        .as_ref()
        .and_then(|e| exif_string(e, Tag::ImageDescription))
        .filter(|s| !s.is_empty())
        .or_else(|| Some(file_name.replace(['_', '-'], " ")));

    let description = exif
        .as_ref()
        .and_then(|e| exif_string(e, Tag::UserComment))
        .filter(|s| !s.is_empty());

    let created = exif
        .as_ref()
        .and_then(|e| exif_datetime(e, Tag::DateTimeOriginal))
        .or_else(|| exif.as_ref().and_then(|e| exif_datetime(e, Tag::DateTime)))
        .or_else(|| datetime_from_filename(&file_name))
        .or_else(|| file_created_datetime(path));

    Some(GalleryImage {
        src,
        thumb_src: None,
        width,
        height,
        title,
        description,
        created,
    })
}

fn read_exif(path: &Path) -> anyhow::Result<Exif> {
    let file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut bufreader = std::io::BufReader::new(&file);
    Reader::new()
        .read_from_container(&mut bufreader)
        .with_context(|| format!("read exif from {}", path.display()))
}

fn image_dimensions_from_exif(exif: &Exif) -> Option<(u32, u32)> {
    fn parse_u32(exif: &Exif, tag: Tag) -> Option<u32> {
        match &exif.get_field(tag, In::PRIMARY)?.value {
            Value::Short(v) => v.first().copied().map(u32::from),
            Value::Long(v) => v.first().copied(),
            _ => None,
        }
    }

    let width = parse_u32(exif, Tag::ImageWidth)
        .or_else(|| parse_u32(exif, Tag::PixelXDimension))?;
    let height = parse_u32(exif, Tag::ImageLength)
        .or_else(|| parse_u32(exif, Tag::PixelYDimension))?;
    Some((width, height))
}

fn exif_string(exif: &Exif, tag: Tag) -> Option<String> {
    let field = exif.get_field(tag, In::PRIMARY)?;
    match &field.value {
        Value::Ascii(v) => v
            .first()
            .and_then(|bytes| String::from_utf8(bytes.clone()).ok())
            .filter(|s| !s.is_empty()),
        _ => None,
    }
}

fn exif_datetime(exif: &Exif, tag: Tag) -> Option<crate::time::UtcDateTime> {
    let s = exif_string(exif, tag)?;
    parse_exif_datetime(&s)
}

fn datetime_from_filename(name: &str) -> Option<crate::time::UtcDateTime> {
    // Try patterns like:
    // ffxiv_20250820_205257_085.png -> 2025-08-20 20:52:57
    // IMG_20231001_123456.jpg -> 2023-10-01 12:34:56
    // Screenshot_2023-10-01_12-34-56.png
    // 2023-10-01_12-34-56.png
    let patterns = [
        regex::Regex::new(r"(\d{4})(\d{2})(\d{2})_(\d{2})(\d{2})(\d{2})").unwrap(),
        regex::Regex::new(r"(\d{4})-(\d{2})-(\d{2})[ _](\d{2})[-:]?(\d{2})[-:]?(\d{2})").unwrap(),
    ];

    for pattern in &patterns {
        if let Some(caps) = pattern.captures(name) {
            let year: i32 = caps.get(1)?.as_str().parse().ok()?;
            let month: u8 = caps.get(2)?.as_str().parse().ok()?;
            let day: u8 = caps.get(3)?.as_str().parse().ok()?;
            let hour: u8 = caps.get(4)?.as_str().parse().ok()?;
            let minute: u8 = caps.get(5)?.as_str().parse().ok()?;
            let second: u8 = caps.get(6)?.as_str().parse().ok()?;
            return build_datetime(year, month, day, hour, minute, second);
        }
    }
    None
}

fn file_created_datetime(path: &Path) -> Option<crate::time::UtcDateTime> {
    let metadata = std::fs::metadata(path).ok()?;
    let created = metadata.created().ok()?;
    let duration = created.duration_since(std::time::UNIX_EPOCH).ok()?;
    crate::time::UtcDateTime::from_unix_timestamp(duration.as_secs() as i64).ok()
}

fn build_datetime(
    year: i32,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
) -> Option<crate::time::UtcDateTime> {
    let date = crate::time::Date::from_calendar_date(year, month.try_into().ok()?, day).ok()?;
    let time = crate::time::Time::from_hms(hour, minute, second).ok()?;
    Some(crate::time::UtcDateTime::new(date, time))
}

fn parse_exif_datetime(s: &str) -> Option<crate::time::UtcDateTime> {
    // EXIF DateTime format: "2023:10:01 12:34:56"
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }
    let date_parts: Vec<&str> = parts[0].split(':').collect();
    let time_parts: Vec<&str> = parts[1].split(':').collect();
    if date_parts.len() != 3 || time_parts.len() != 3 {
        return None;
    }

    let year: i32 = date_parts[0].parse().ok()?;
    let month: u8 = date_parts[1].parse().ok()?;
    let day: u8 = date_parts[2].parse().ok()?;
    let hour: u8 = time_parts[0].parse().ok()?;
    let minute: u8 = time_parts[1].parse().ok()?;
    let second: u8 = time_parts[2].parse().ok()?;

    build_datetime(year, month, day, hour, minute, second)
}

impl quote::ToTokens for GalleryImage {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            src,
            thumb_src,
            width,
            height,
            title,
            description,
            created,
        } = self;

        let thumb_src_tokens = match thumb_src {
            Some(t) => quote::quote! { Some(#t.to_string()) },
            None => quote::quote! { None },
        };

        let title_tokens = match title {
            Some(t) => quote::quote! { Some(#t.to_string()) },
            None => quote::quote! { None },
        };

        let desc_tokens = match description {
            Some(d) => quote::quote! { Some(#d.to_string()) },
            None => quote::quote! { None },
        };

        let created_tokens = match created {
            Some(dt) => {
                let ts = dt.unix_timestamp();
                quote::quote! {
                    Some(aoike::time::UtcDateTime::from_unix_timestamp(#ts).unwrap())
                }
            }
            None => quote::quote! { None },
        };

        tokens.extend(quote::quote! {
            aoike::GalleryImage {
                src: #src.to_string(),
                thumb_src: #thumb_src_tokens,
                width: #width,
                height: #height,
                title: #title_tokens,
                description: #desc_tokens,
                created: #created_tokens,
            }
        });
    }
}

fn date_tokens(date: Option<&Date>) -> proc_macro2::TokenStream {
    match date {
        Some(d) => {
            let year = d.year();
            let month = u8::from(d.month());
            let day = d.day();
            quote::quote! {
                Some(aoike::time::Date::from_calendar_date(
                    #year,
                    aoike::time::Month::try_from(#month).unwrap(),
                    #day,
                ).unwrap())
            }
        }
        None => quote::quote! { None },
    }
}

impl quote::ToTokens for GalleryGroup {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            name,
            slug,
            description_html,
            images,
        } = self;

        let desc_tokens = match description_html {
            Some(d) => quote::quote! { Some(#d.to_string()) },
            None => quote::quote! { None },
        };

        tokens.extend(quote::quote! {
            aoike::GalleryGroup {
                name: #name.to_string(),
                slug: #slug.to_string(),
                description_html: #desc_tokens,
                images: vec![#(#images),*],
            }
        });
    }
}

impl quote::ToTokens for GalleryTimelineItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::DateGroup {
                date,
                loose_image_indices,
                folder_group_indices,
            } => {
                let date_tok = date_tokens(date.as_ref());
                tokens.extend(quote::quote! {
                    aoike::GalleryTimelineItem::DateGroup {
                        date: #date_tok,
                        loose_image_indices: vec![#(#loose_image_indices),*],
                        folder_group_indices: vec![#(#folder_group_indices),*],
                    }
                });
            }
        }
    }
}

impl quote::ToTokens for GalleryCategory {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            name,
            slug,
            description_html,
            loose_images,
            groups,
            timeline,
        } = self;

        let desc_tokens = match description_html {
            Some(d) => quote::quote! { Some(#d.to_string()) },
            None => quote::quote! { None },
        };

        tokens.extend(quote::quote! {
            aoike::GalleryCategory {
                name: #name.to_string(),
                slug: #slug.to_string(),
                description_html: #desc_tokens,
                loose_images: vec![#(#loose_images),*],
                groups: vec![#(#groups),*],
                timeline: vec![#(#timeline),*],
            }
        });
    }
}

pub fn generate_gallery_code(categories: Vec<GalleryCategory>) -> String {
    let token = quote::quote! {
        pub fn gallery() -> &'static [aoike::GalleryCategory] {
            static GALLERY: std::sync::LazyLock<Vec<aoike::GalleryCategory>> = std::sync::LazyLock::new(|| {
                let mut categories: Vec<aoike::GalleryCategory> = vec![#(#categories),*];
                categories.sort_by(|a, b| a.name.cmp(&b.name));
                categories
            });
            &GALLERY
        }
    };

    prettyplease::unparse(&syn::parse_quote! {
        #token
    })
}
