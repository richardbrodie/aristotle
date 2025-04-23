mod book;
mod content;
mod error;
mod html;
mod index;
mod manifest;
mod metadata;
mod spine;
mod zip;

pub use book::Book;
pub use error::Error;

pub use content::Content;
pub use html::ElementVariant;
pub use html::Node;

pub trait Indexable {
    fn content(&mut self, id: &str) -> Result<Content, Error>;
    fn first(&mut self) -> Result<Content, Error>;
    fn next(&mut self, cur: &str) -> Result<Content, Error>;
    fn prev(&mut self, cur: &str) -> Result<Content, Error>;
}
