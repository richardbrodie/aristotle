use std::path::PathBuf;

use crate::manifest::ManifestItem;

#[derive(Debug, Default)]
pub enum MediaType {
    Image,
    Xhtml,
    Css,
    Font,
    Toc,
    #[default]
    Unknown,
}
impl From<&str> for MediaType {
    fn from(value: &str) -> Self {
        match value {
            "application/xhtml+xml" => MediaType::Xhtml,
            "application/x-font-truetype" => MediaType::Font,
            "text/css" => MediaType::Css,
            "application/x-dtbncx+xml" => MediaType::Toc,
            "image/jpeg" => MediaType::Image,
            _ => MediaType::Unknown,
        }
    }
}

#[derive(Debug, Default)]
pub struct Document {
    id: String,
    mediatype: MediaType,
    path: PathBuf,
}
impl From<&ManifestItem> for Document {
    fn from(value: &ManifestItem) -> Self {
        Document {
            id: value.id().to_owned(),
            mediatype: value.mediatype(),
            path: value.href().to_owned(),
        }
    }
}
