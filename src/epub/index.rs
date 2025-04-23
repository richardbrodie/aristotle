use crate::manifest::Manifest;
use crate::spine::Spine;
use crate::{Content, Element};

struct Index {
    manifest: Manifest,
    spine: Spine,
}
impl Index {
    pub fn items(&self) -> impl Iterator<Item = Element> + '_ {
        self.spine
            .items()
            .map_while(move |id| self.manifest.find(id))
            .map(Element::new)
    }
    pub fn element(&self, id: &str) -> Option<Element> {
        self.manifest.find(id).map(Element::new)
    }
    pub fn next_item(&self, id: &str) -> Option<Element> {
        self.spine
            .next(id)
            .and_then(|i| self.manifest.find(i))
            .map(Element::new)
    }
    //pub fn content(&mut self, id: &str) -> Option<Content> {
    //    if let Some(item) = self.manifest.find(id) {
    //        let el = Element::new(item);
    //        if let Ok(bytes) = self.read_document(el.path().unwrap()) {
    //            return Some(Content::new(bytes));
    //        }
    //    }
    //    None
    //}
}
