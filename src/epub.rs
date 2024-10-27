mod book;
mod content;
mod element;
mod guide;
//mod index;
mod manifest;
mod metadata;
mod spine;

pub use content::Content;
use std::borrow::Cow;

pub use book::Book;
pub use element::Element;
use quick_xml::events::attributes::AttrError;

#[derive(Debug)]
pub enum EpubError {
    XmlDocument(quick_xml::Error),
    XmlField(String),
    XmlAttribute,
    File,
    Zip,
    StringParse,
    Content,
}
impl From<quick_xml::Error> for EpubError {
    fn from(error: quick_xml::Error) -> Self {
        Self::XmlDocument(error)
    }
}

impl From<AttrError> for EpubError {
    fn from(error: AttrError) -> Self {
        Self::XmlDocument(quick_xml::Error::InvalidAttr(error))
    }
}

fn cow_to_string(c: Cow<[u8]>) -> Result<String, EpubError> {
    String::from_utf8(c.into_owned()).map_err(|_| EpubError::StringParse)
}
fn text_to_string(c: Result<Cow<str>, quick_xml::Error>) -> Result<String, EpubError> {
    c.map(|i| i.into_owned())
        .map_err(|_| EpubError::StringParse)
}

#[derive(Debug, Default, Clone, Copy)]
pub enum TextStyle {
    #[default]
    Regular,
    Italic,
    Bold,
}

#[derive(Debug, Default, Clone)]
pub struct TextElement {
    pub style: TextStyle,
    pub content: String,
}

#[derive(Debug)]
pub enum ContentElement {
    Heading(TextElement),
    Paragraph,
    Inline(TextElement),
    Linebreak,
}
