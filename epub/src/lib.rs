use quick_xml::events::Event;

mod book;
mod element;
mod guide;
mod index;
mod manifest;
mod metadata;
mod spine;

#[derive(Debug)]
pub enum Error {
    XmlDocument(quick_xml::Error),
    XmlField(String),
    XmlAttribute,
    File,
    Zip,
    StringParse,
    Content,
}
impl From<quick_xml::Error> for Error {
    fn from(error: quick_xml::Error) -> Self {
        Self::XmlDocument(error)
    }
}

impl From<AttrError> for Error {
    fn from(error: AttrError) -> Self {
        Self::XmlDocument(quick_xml::Error::InvalidAttr(error))
    }
}

#[derive(Debug, Default)]
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
    pub fn content(&self) -> Option<&str> {
        let mut reader = Reader::from_str(&self.content);
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"body" => {
                    let c = reader.read_to_end(e.name()).unwrap();
                    let start = c.start as usize;
                    let end = c.end as usize;
                    return Some(&self.content[start..end]);
                }
                Err(_) => return None,
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }
        return None;
    }
}

fn cow_to_string(c: Cow<[u8]>) -> Result<String, Error> {
    String::from_utf8(c.into_owned()).map_err(|_| Error::StringParse)
}
fn text_to_string(c: Result<Cow<str>, quick_xml::Error>) -> Result<String, Error> {
    c.map(|i| i.into_owned()).map_err(|_| Error::StringParse)
}

use std::borrow::Cow;
use std::usize;

pub use book::Book;
pub use element::Element;
use quick_xml::events::attributes::AttrError;
use quick_xml::Reader;
