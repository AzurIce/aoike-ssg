#[cfg(feature = "build")]
pub mod build;

pub use time;
use time::{Date, UtcDateTime};

#[derive(Clone, PartialEq)]
pub struct PostData {
    pub title: String,
    pub slug: String,
    pub summary_html: String,
    pub content_html: String,
    pub created: UtcDateTime,
    pub updated: UtcDateTime,
}

#[derive(Clone)]
pub struct Site {
    pub posts: &'static [PostData],
    pub index: &'static PostData,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GalleryImage {
    pub src: String,
    pub thumb_src: Option<String>,
    pub width: u32,
    pub height: u32,
    pub title: Option<String>,
    pub description: Option<String>,
    pub created: Option<UtcDateTime>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GalleryCategory {
    pub name: String,
    pub slug: String,
    pub images: Vec<GalleryImage>,
    pub date_groups: Vec<(Option<Date>, Vec<usize>)>,
}
