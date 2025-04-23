mod book;
mod document;
mod guide;
mod manifest;
mod metadata;
mod spine;

#[derive(Debug)]
pub enum Error {
    Xml,
    File,
    Zip,
}

pub use book::Book;
pub use document::Document;
