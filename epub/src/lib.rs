mod book;
mod element;
mod guide;
mod manifest;
mod metadata;
mod spine;

#[derive(Debug)]
pub enum Error {
    Xml,
    File,
    Zip,
    StringParse,
}
impl From<FromUtf8Error> for Error {
    fn from(value: FromUtf8Error) -> Self {
        Self::StringParse
    }
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
}

use std::string::FromUtf8Error;

pub use book::Book;
pub use element::Element;
