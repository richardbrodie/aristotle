use std::path::Path;

use crate::text::FontError;

use super::{face::Face, index::IndexedFont, style::FontStyle};

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

    pub fn face(&self, style: FontStyle) -> Result<&Face, FontError> {
        self.faces
            .iter()
            .find(|s| s.style() == style)
            .ok_or(FontError::MissingFace)
    }
    pub fn face_styles(&self) -> impl Iterator<Item = FontStyle> + use<'_> {
        self.faces.iter().map(|f| f.style())
    }
}
