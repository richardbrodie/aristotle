use std::path::{Path, PathBuf};

use crate::element::MediaType;
use crate::Error;

#[derive(Debug, Default)]
pub struct Manifest {
    items: Vec<ManifestItem>,
}
impl Manifest {
    pub fn extract(doc: &roxmltree::Document<'_>) -> Result<Manifest, Error> {
        let manifest_node = doc
            .descendants()
            .find(|n| n.has_tag_name("manifest"))
            .unwrap();

        let items: Vec<_> = manifest_node
            .children()
            .filter(|node| node.has_tag_name("item"))
            .map(|node| {
                let mut href = String::new();
                let mut mediatype = String::new();
                let mut id = String::new();
                for attr in node.attributes() {
                    match attr.name() {
                        "href" => href = attr.value().to_owned(),
                        "id" => id = attr.value().to_owned(),
                        "media-type" => mediatype = attr.value().to_owned(),
                        _ => {}
                    }
                }
                ManifestItem::new(mediatype, id, href)
            })
            .collect();
        Ok(Manifest { items })
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
