use std::path::Path;

use anyhow::Context;
use exif::{Exif, In, Reader, Tag, Value};
use walkdir::WalkDir;

use crate::{time::Date, GalleryCategory, GalleryImage};

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

        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unnamed")
            .to_string();
        let slug = slug::slugify(&name);

        let mut images: Vec<GalleryImage> = WalkDir::new(&path)
            .into_iter()
            .flatten()
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| parse_image(e.path(), dir, &slug))
            .collect();

        images.sort_by(|a, b| b.created.cmp(&a.created));

        let date_groups = group_images_by_date(&images);

        categories.push(GalleryCategory {
            name,
            slug,
            images,
            date_groups,
        });
    }

    categories.sort_by(|a, b| a.name.cmp(&b.name));
    categories
}

fn group_images_by_date(images: &[GalleryImage]) -> Vec<(Option<Date>, Vec<usize>)> {
    use std::collections::BTreeMap;

    let mut known: BTreeMap<Date, Vec<usize>> = BTreeMap::new();
    let mut unknown: Vec<usize> = Vec::new();

    for (idx, image) in images.iter().enumerate() {
        match image.created {
            Some(dt) => known.entry(dt.date()).or_default().push(idx),
            None => unknown.push(idx),
        }
    }

    for indices in known.values_mut() {
        indices.sort_by(|&a, &b| images[b].created.cmp(&images[a].created));
    }

    let mut result: Vec<_> = known
        .into_iter()
        .map(|(date, indices)| (Some(date), indices))
        .collect();
    result.sort_by(|a, b| b.0.cmp(&a.0));

    if !unknown.is_empty() {
        result.push((None, unknown));
    }

    result
}

fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| IMAGE_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn parse_image(path: &Path, _gallery_dir: &Path, category_slug: &str) -> Option<GalleryImage> {
    if !is_image(path) {
        return None;
    }

    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image")
        .to_string();

    let src = format!("/static/gallery/{}/{}", category_slug, path.file_name()?.to_str()?);

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

impl quote::ToTokens for GalleryCategory {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            name,
            slug,
            images,
            date_groups,
        } = self;

        let group_tokens: Vec<_> = date_groups
            .iter()
            .map(|(date, indices)| {
                let date_tok = date_tokens(date.as_ref());
                quote::quote! { (#date_tok, vec![#(#indices),*]) }
            })
            .collect();

        tokens.extend(quote::quote! {
            aoike::GalleryCategory {
                name: #name.to_string(),
                slug: #slug.to_string(),
                images: vec![#(#images),*],
                date_groups: vec![#(#group_tokens),*],
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
