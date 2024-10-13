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
    _content: String,
}
impl Content {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            _content: String::from_utf8(bytes).unwrap(),
        }
    }
}

pub use book::Book;
pub use element::Element;
