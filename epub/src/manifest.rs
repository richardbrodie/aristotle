use std::path::{Path, PathBuf};

use crate::element::MediaType;

#[derive(Debug, Default)]
pub struct Manifest {
    items: Vec<ManifestItem>,
}
impl Manifest {
    pub fn new(items: Vec<ManifestItem>) -> Self {
        Self { items }
    }
    pub fn find(&self, id: &str) -> Option<&ManifestItem> {
        self.items.iter().find(|i| i.id == id)
    }
}

#[derive(Debug, Default)]
pub struct ManifestItem {
    id: String,
    href: PathBuf,
    mediatype: String,
}
impl ManifestItem {
    pub fn new(mediatype: String, id: String, href: String) -> Self {
        Self {
            id,
            href: PathBuf::from(href),
            mediatype,
        }
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn href(&self) -> &Path {
        Path::new(&self.href)
    }
    pub fn mediatype(&self) -> MediaType {
        self.mediatype.as_str().into()
    }
}
