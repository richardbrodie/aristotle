mod face;
mod index;

use std::path::Path;

pub use index::{FontIndexer, Indexer};

use self::face::Face;
use self::index::IndexedFont;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum FontStyle {
    #[default]
    Regular,
    Bold,
    Italic,
    BoldItalic,
    Mono,
}
impl From<&str> for FontStyle {
    fn from(value: &str) -> Self {
        match value {
            "Regular" => Self::Regular,
            "Bold" => Self::Bold,
            "Italic" => Self::Italic,
            "Bold Italic" => Self::BoldItalic,
            "Mono" | "Book" => Self::Mono,
            _ => Self::Regular,
        }
    }
}

pub trait Faces {
    fn get_face(&self, style: FontStyle) -> Option<&Face>;
    fn styles(&self) -> impl Iterator<Item = FontStyle>;
}

#[derive(Debug, Default, Clone)]
pub struct Family {
    pub name: String,
    pub faces: Vec<Face>,
}
impl Family {
    pub fn from_font<P>(p: P) -> Self
    where
        P: AsRef<Path>,
    {
        let i = IndexedFont::new(p);
        let f = Face::new(&i);
        Family {
            name: i.family,
            faces: vec![f],
        }
    }
}

impl Faces for Family {
    fn get_face(&self, style: FontStyle) -> Option<&Face> {
        self.faces.iter().find(|s| s.style() == style)
    }
    fn styles(&self) -> impl Iterator<Item = FontStyle> {
        self.faces.iter().map(|f| f.style())
    }
}
