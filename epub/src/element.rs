use std::path::{Path, PathBuf};

use crate::manifest::ManifestItem;

#[derive(Debug, Default)]
pub enum MediaType {
    Image,
    XHTML,
    CSS,
    Font,
    TOC,
    #[default]
    Unknown,
}
impl From<&str> for MediaType {
    fn from(value: &str) -> Self {
        match value {
            "application/xhtml+xml" => MediaType::XHTML,
            "application/x-font-truetype" => MediaType::Font,
            "text/css" => MediaType::CSS,
            "application/x-dtbncx+xml" => MediaType::TOC,
            "image/jpeg" => MediaType::Image,
            _ => MediaType::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Element {
    id: String,
    title: Option<String>,
    mediatype: MediaType,
    path: PathBuf,
}
impl Element {
    pub fn new(value: &ManifestItem) -> Self {
        Element {
            id: value.id().to_owned(),
            title: None,
            mediatype: value.mediatype(),
            path: value.href().to_owned(),
        }
    }
    pub fn path(&self) -> Option<&str> {
        self.path.to_str()
    }
}
