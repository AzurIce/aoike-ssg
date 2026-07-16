use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use exif::{Exif, In, Reader, Tag, Value};
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use serde::Deserialize;
use walkdir::WalkDir;

use crate::{GalleryCategory, GalleryGroup, GalleryImage, GalleryTimelineItem, time::Date};

const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif"];
const PATH_SEGMENT_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'/')
    .add(b':')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GalleryBuildMode {
    Embed,
    ShadowTos,
}

pub struct GalleryBuildOutput {
    pub categories: Vec<GalleryCategory>,
    pub trunk_assets: String,
}

pub fn build_gallery(dir: impl AsRef<Path>, mode: GalleryBuildMode) -> Result<GalleryBuildOutput> {
    let dir = dir.as_ref();
    let resolver = GalleryUrlResolver::new(dir, mode)?;
    let categories = parse_gallery_with_resolver(dir, &resolver)?;
    let trunk_assets = match mode {
        GalleryBuildMode::Embed => format!(
            r#"<link rel="copy-dir" href="{}/" data-trunk>"#,
            path_to_url(dir)?
        ),
        GalleryBuildMode::ShadowTos => String::new(),
    };
    Ok(GalleryBuildOutput {
        categories,
        trunk_assets,
    })
}

pub fn parse_gallery(dir: impl AsRef<Path>) -> Vec<GalleryCategory> {
    let dir = dir.as_ref();
    if !dir.exists() {
        return Vec::new();
    }

    let resolver = GalleryUrlResolver::embedded_with_prefix(dir, "/static/gallery".to_string());
    parse_gallery_with_resolver(dir, &resolver)
        .expect("embedded gallery URL generation must succeed")
}

fn parse_gallery_with_resolver(
    dir: &Path,
    resolver: &GalleryUrlResolver,
) -> Result<Vec<GalleryCategory>> {
    if !dir.exists() {
        bail!("gallery source directory does not exist: {}", dir.display());
    }

    let mut categories = Vec::new();

    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        if let Some(category) = parse_category(&path, resolver)? {
            categories.push(category);
        }
    }

    categories.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(categories)
}

fn parse_category(path: &Path, resolver: &GalleryUrlResolver) -> Result<Option<GalleryCategory>> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unnamed")
        .to_string();
    let slug = slug::slugify(&name);
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
            if let Some(group) = parse_group(&entry_path, resolver)? {
                if !group.images.is_empty() {
                    groups.push(group);
                }
            }
        } else if is_image(&entry_path) {
            if let Some(image) = parse_image(&entry_path, resolver)? {
                loose_images.push(image);
            }
        }
    }

    loose_images.sort_by(|a, b| b.created.cmp(&a.created));

    let timeline = build_timeline(&loose_images, &groups);

    Ok(Some(GalleryCategory {
        name,
        slug,
        description_html,
        loose_images,
        groups,
        timeline,
    }))
}

fn parse_group(path: &Path, resolver: &GalleryUrlResolver) -> Result<Option<GalleryGroup>> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unnamed")
        .to_string();
    let slug = slug::slugify(&name);
    let description_html = path
        .join("index.md")
        .exists()
        .then(|| parse_markdown_file(&path.join("index.md")))
        .flatten();

    let images: Vec<GalleryImage> = WalkDir::new(path)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file() && is_image(e.path()))
        .map(|e| parse_image(e.path(), resolver))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .collect();

    // Preserve filesystem order within a group; don't sort by time.
    // (If the user wants a specific order, they can rename files.)
    Ok(Some(GalleryGroup {
        name,
        slug,
        description_html,
        images,
    }))
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
        by_date
            .entry(date)
            .or_default()
            .loose_image_indices
            .push(idx);
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
        by_date
            .entry(date)
            .or_default()
            .folder_group_indices
            .push(group_idx);
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

fn parse_image(path: &Path, resolver: &GalleryUrlResolver) -> Result<Option<GalleryImage>> {
    if !is_image(path) {
        return Ok(None);
    }

    let file_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image")
        .to_string();

    let src = resolver.resolve(path)?;

    let exif = read_exif(path).ok();
    let (width, height) = exif
        .as_ref()
        .and_then(image_dimensions_from_exif)
        .or_else(|| {
            imagesize::size(path)
                .ok()
                .map(|s| (s.width as u32, s.height as u32))
        })
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

    let offset_original = exif
        .as_ref()
        .and_then(|e| exif_string(e, Tag::OffsetTimeOriginal));
    let offset_datetime = exif.as_ref().and_then(|e| exif_string(e, Tag::OffsetTime));

    let created = exif
        .as_ref()
        .and_then(|e| exif_datetime(e, Tag::DateTimeOriginal, offset_original.as_deref()))
        .or_else(|| {
            exif.as_ref()
                .and_then(|e| exif_datetime(e, Tag::DateTime, offset_datetime.as_deref()))
        })
        .or_else(|| datetime_from_filename(&file_name))
        .or_else(|| file_created_datetime(path));

    let (rating, label) = read_xmp_metadata(path);

    Ok(Some(GalleryImage {
        src,
        thumb_src: None,
        width,
        height,
        title,
        description,
        created,
        rating,
        label,
    }))
}

struct GalleryUrlResolver {
    source_root: PathBuf,
    kind: GalleryUrlResolverKind,
}

enum GalleryUrlResolverKind {
    Embedded { public_prefix: String },
    ShadowTos(ShadowTosResolver),
}

impl GalleryUrlResolver {
    fn new(source_root: &Path, mode: GalleryBuildMode) -> Result<Self> {
        match mode {
            GalleryBuildMode::Embed => Self::embedded(source_root),
            GalleryBuildMode::ShadowTos => Ok(Self {
                source_root: source_root.to_path_buf(),
                kind: GalleryUrlResolverKind::ShadowTos(ShadowTosResolver::discover(source_root)?),
            }),
        }
    }

    fn embedded(source_root: &Path) -> Result<Self> {
        let directory_name = source_root
            .file_name()
            .context("gallery source directory has no name")?
            .to_str()
            .context("gallery source directory name is not UTF-8")?;
        Ok(Self::embedded_with_prefix(
            source_root,
            format!("/{}", encode_path_segment(directory_name)),
        ))
    }

    fn embedded_with_prefix(source_root: &Path, public_prefix: String) -> Self {
        Self {
            source_root: source_root.to_path_buf(),
            kind: GalleryUrlResolverKind::Embedded { public_prefix },
        }
    }

    fn resolve(&self, path: &Path) -> Result<String> {
        match &self.kind {
            GalleryUrlResolverKind::Embedded { public_prefix } => {
                let relative = path.strip_prefix(&self.source_root).with_context(|| {
                    format!(
                        "gallery image {} is outside {}",
                        path.display(),
                        self.source_root.display()
                    )
                })?;
                Ok(format!("{}/{}", public_prefix, path_to_url(relative)?))
            }
            GalleryUrlResolverKind::ShadowTos(resolver) => resolver.resolve(path),
        }
    }
}

#[derive(Deserialize)]
struct ShadowConfig {
    name: String,
    backend: ShadowBackendConfig,
}

#[derive(Deserialize)]
struct ShadowBackendConfig {
    #[serde(rename = "type")]
    kind: String,
    endpoint: String,
    bucket: String,
    #[serde(default)]
    prefix: String,
}

#[derive(Deserialize)]
struct ShadowRefDocument {
    oid: String,
}

struct ShadowTosResolver {
    repository_root: PathBuf,
    refs_root: PathBuf,
    public_base_url: String,
    object_prefix: String,
}

impl ShadowTosResolver {
    fn discover(source_root: &Path) -> Result<Self> {
        let source_root = source_root
            .canonicalize()
            .with_context(|| format!("failed to resolve gallery path {}", source_root.display()))?;
        let repository_root = source_root
            .ancestors()
            .find(|ancestor| ancestor.join("shadow.toml").is_file())
            .context("could not find shadow.toml above gallery source directory")?
            .to_path_buf();
        let config_path = repository_root.join("shadow.toml");
        let config: ShadowConfig = toml::from_str(
            &std::fs::read_to_string(&config_path)
                .with_context(|| format!("failed to read {}", config_path.display()))?,
        )
        .with_context(|| format!("failed to parse {}", config_path.display()))?;
        if config.backend.kind != "volcengine_tos" {
            bail!("ShadowTos requires a volcengine_tos backend");
        }

        let public_base_url =
            tos_public_base_url(&config.backend.endpoint, &config.backend.bucket)?;
        let object_prefix = config
            .backend
            .prefix
            .trim_matches('/')
            .split('/')
            .filter(|part| !part.is_empty())
            .chain(std::iter::once(config.name.as_str()))
            .map(encode_path_segment)
            .collect::<Vec<_>>()
            .join("/");

        Ok(Self {
            refs_root: repository_root.join(".shadow").join("refs"),
            repository_root,
            public_base_url,
            object_prefix,
        })
    }

    fn resolve(&self, path: &Path) -> Result<String> {
        let absolute = path
            .canonicalize()
            .with_context(|| format!("failed to resolve gallery image {}", path.display()))?;
        let relative = absolute
            .strip_prefix(&self.repository_root)
            .with_context(|| {
                format!(
                    "gallery image {} is outside the Shadow repository",
                    path.display()
                )
            })?;
        let mut ref_path = self.refs_root.join(relative);
        let file_name = ref_path
            .file_name()
            .context("gallery image has no file name")?
            .to_string_lossy();
        ref_path.set_file_name(format!("{file_name}.ref"));
        let reference: ShadowRefDocument = toml::from_str(
            &std::fs::read_to_string(&ref_path)
                .with_context(|| format!("failed to read Shadow ref {}", ref_path.display()))?,
        )
        .with_context(|| format!("failed to parse Shadow ref {}", ref_path.display()))?;
        let oid = reference
            .oid
            .strip_prefix("sha256:")
            .context("Shadow ref object ID must start with sha256:")?;
        if oid.len() != 64 || !oid.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            bail!("invalid SHA-256 object ID in {}", ref_path.display());
        }

        Ok(format!(
            "{}/{}/objects/sha256/{}/{}",
            self.public_base_url,
            self.object_prefix,
            &oid[..2],
            &oid[2..]
        ))
    }
}

fn tos_public_base_url(endpoint: &str, bucket: &str) -> Result<String> {
    let endpoint = endpoint.trim_end_matches('/');
    let (scheme, authority) = endpoint
        .split_once("://")
        .context("TOS endpoint must include a URL scheme")?;
    if authority.is_empty() || authority.contains('/') {
        bail!("TOS endpoint must not contain a path");
    }
    Ok(format!("{}://{}.{}", scheme, bucket, authority))
}

fn path_to_url(path: &Path) -> Result<String> {
    path.components()
        .map(|component| {
            component
                .as_os_str()
                .to_str()
                .context("gallery path is not UTF-8")
                .map(encode_path_segment)
        })
        .collect::<Result<Vec<_>>>()
        .map(|components| components.join("/"))
}

fn encode_path_segment(segment: &str) -> String {
    utf8_percent_encode(segment, PATH_SEGMENT_ENCODE_SET).to_string()
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

    let width =
        parse_u32(exif, Tag::ImageWidth).or_else(|| parse_u32(exif, Tag::PixelXDimension))?;
    let height =
        parse_u32(exif, Tag::ImageLength).or_else(|| parse_u32(exif, Tag::PixelYDimension))?;
    Some((width, height))
}

const XMP_SIGNATURE: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";

fn read_xmp_metadata(path: &Path) -> (Option<u8>, Option<String>) {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return (None, None),
    };
    let packet = match extract_xmp_packet(&bytes) {
        Some(p) => p,
        None => return (None, None),
    };
    let text = String::from_utf8_lossy(&packet);
    (parse_xmp_rating(&text), parse_xmp_label(&text))
}

fn extract_xmp_packet(jpeg_bytes: &[u8]) -> Option<Vec<u8>> {
    if jpeg_bytes.len() < 2 || jpeg_bytes[0] != 0xFF || jpeg_bytes[1] != 0xD8 {
        return None;
    }
    let mut i = 2usize;
    while i + 4 < jpeg_bytes.len() {
        if jpeg_bytes[i] != 0xFF {
            return None;
        }
        let marker = jpeg_bytes[i + 1];
        if marker == 0xD9 || marker == 0xD8 || marker == 0x01 || (0xD0..=0xD7).contains(&marker) {
            i += 2;
            continue;
        }
        let len = u16::from_be_bytes([jpeg_bytes[i + 2], jpeg_bytes[i + 3]]) as usize;
        let segment_end = i + 2 + len;
        if segment_end > jpeg_bytes.len() {
            return None;
        }
        if marker == 0xE1 && len > XMP_SIGNATURE.len() {
            let payload_start = i + 4;
            let sig_end = payload_start + XMP_SIGNATURE.len();
            if sig_end <= segment_end && &jpeg_bytes[payload_start..sig_end] == XMP_SIGNATURE {
                return Some(jpeg_bytes[sig_end..segment_end].to_vec());
            }
        }
        i = segment_end;
    }
    None
}

fn parse_xmp_rating(xmp_text: &str) -> Option<u8> {
    let re = regex::Regex::new(r"<xmp:Rating[^>]*>([^<]+)</xmp:Rating>").ok()?;
    re.captures(xmp_text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim())
        .and_then(|s| s.parse::<u8>().ok())
        .filter(|&r| r > 0)
}

fn parse_xmp_label(xmp_text: &str) -> Option<String> {
    // Match both <xmp:Label>Red</xmp:Label> and <xmp:Label rdf:parseType="Resource">...</xmp:Label>
    // For our use case a simple tag content extraction is sufficient.
    let re = regex::Regex::new(r"<xmp:Label[^>]*>([^<]+)</xmp:Label>").ok()?;
    re.captures(xmp_text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim().to_string())
        .filter(|s| !s.is_empty())
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

fn exif_datetime(exif: &Exif, tag: Tag, offset: Option<&str>) -> Option<crate::time::UtcDateTime> {
    let s = exif_string(exif, tag)?;
    parse_exif_datetime(&s, offset)
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
    // Prefer modification time: it is usually preserved when files are copied,
    // whereas creation time on Windows is often refreshed to the copy time.
    let time = metadata
        .modified()
        .ok()
        .or_else(|| metadata.created().ok())?;
    let duration = time.duration_since(std::time::UNIX_EPOCH).ok()?;
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

fn parse_exif_datetime(s: &str, offset: Option<&str>) -> Option<crate::time::UtcDateTime> {
    // EXIF DateTime format is "2023:10:01 12:34:56", but some tools write
    // "2023-10-01 12:34:56". Accept both ':' and '-' as date separators.
    let s = s.trim();
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() != 2 {
        return None;
    }

    let date_delim = if parts[0].contains(':') { ':' } else { '-' };
    let date_parts: Vec<&str> = parts[0].split(date_delim).collect();
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

    let date = crate::time::Date::from_calendar_date(year, month.try_into().ok()?, day).ok()?;
    let time = crate::time::Time::from_hms(hour, minute, second).ok()?;
    let local_dt = crate::time::PrimitiveDateTime::new(date, time);

    let offset = offset
        .and_then(parse_exif_offset)
        .unwrap_or(crate::time::UtcOffset::UTC);
    let utc_dt = local_dt
        .assume_offset(offset)
        .to_offset(crate::time::UtcOffset::UTC);

    Some(crate::time::UtcDateTime::new(utc_dt.date(), utc_dt.time()))
}

fn parse_exif_offset(s: &str) -> Option<crate::time::UtcOffset> {
    // EXIF offset format: "+08:00", "-05:00", "+00:00", or "Z" for UTC.
    let s = s.trim();
    if s.eq_ignore_ascii_case("Z") || s == "+00:00" {
        return Some(crate::time::UtcOffset::UTC);
    }
    let bytes = s.as_bytes();
    if bytes.len() != 6 || bytes[3] != b':' {
        return None;
    }
    let sign: i8 = match bytes[0] {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let hours: i8 = s[1..3].parse().ok()?;
    let minutes: i8 = s[4..6].parse().ok()?;
    crate::time::UtcOffset::from_hms(sign * hours, sign * minutes, 0).ok()
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
            rating,
            label,
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

        let rating_tokens = match rating {
            Some(r) => quote::quote! { Some(#r) },
            None => quote::quote! { None },
        };

        let label_tokens = match label {
            Some(l) => quote::quote! { Some(#l.to_string()) },
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
                rating: #rating_tokens,
                label: #label_tokens,
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

#[cfg(test)]
mod tests {
    use super::{GalleryUrlResolver, parse_xmp_label, parse_xmp_rating, tos_public_base_url};
    use std::path::Path;

    #[test]
    fn nested_gallery_image_src_keeps_all_subdirectories() {
        let root = Path::new("gallery");
        let image = root
            .join("地球 OL")
            .join("2025-10-25 灵山彗星银河流星")
            .join("银河小延时")
            .join("DSC01730_gallery.jpg");
        let resolver = GalleryUrlResolver::embedded(&root).unwrap();

        assert_eq!(
            resolver.resolve(&image).unwrap(),
            "/gallery/%E5%9C%B0%E7%90%83%20OL/2025-10-25%20%E7%81%B5%E5%B1%B1%E5%BD%97%E6%98%9F%E9%93%B6%E6%B2%B3%E6%B5%81%E6%98%9F/%E9%93%B6%E6%B2%B3%E5%B0%8F%E5%BB%B6%E6%97%B6/DSC01730_gallery.jpg"
        );
    }

    #[test]
    fn builds_virtual_hosted_tos_base_url() {
        assert_eq!(
            tos_public_base_url("https://tos-cn-beijing.volces.com", "azurice-shadow").unwrap(),
            "https://azurice-shadow.tos-cn-beijing.volces.com"
        );
    }

    #[test]
    fn parses_rating_and_label_from_embedded_xmp_text() {
        let xmp = "<xmp:Rating>5</xmp:Rating><xmp:Label>Red</xmp:Label>";
        assert_eq!(parse_xmp_rating(xmp), Some(5));
        assert_eq!(parse_xmp_label(xmp).as_deref(), Some("Red"));
    }
}
