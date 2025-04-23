mod book;
mod element;
mod guide;
mod manifest;
mod metadata;
mod spine;

#[derive(Debug)]
pub enum Error {
    XmlDocument,
    XmlField,
    File,
    Zip,
    StringParse,
}

pub struct Content {
    content: String,
}
impl Content {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            content: String::from_utf8(bytes).unwrap(),
        }
    }
    pub fn raw_str(&self) -> &str {
        self.content.as_str()
    }
    pub fn content(&self) -> Option<String> {
        let doc = roxmltree::Document::parse(&self.content).unwrap();
        let body_node = doc.descendants().find(|n| n.has_tag_name("body")).unwrap();
        body_node.tail().map(ToOwned::to_owned)
    }
}

pub use book::Book;
pub use element::Element;
