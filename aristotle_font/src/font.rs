use crate::indexer::Font;

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
            "BoldItalic" => Self::BoldItalic,
            "Mono" | "Book" => Self::Mono,
            _ => Self::Regular,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Family {
    pub name: String,
    pub faces: Vec<Face>,
}

pub trait Faces {
    fn get_face(&self, style: FontStyle) -> Option<ttf_parser::Face>;
    fn styles(&self) -> impl Iterator<Item = FontStyle>;
}

impl Faces for Family {
    fn get_face(&self, style: FontStyle) -> Option<ttf_parser::Face> {
        self.faces
            .iter()
            .find(|s| s.style == style)
            .map(|f| ttf_parser::Face::parse(&f.bytes, 0).unwrap())
    }
    fn styles(&self) -> impl Iterator<Item = FontStyle> {
        self.faces.iter().map(|f| f.style)
    }
}

#[derive(Default, Clone, Debug)]
pub struct Face {
    pub bytes: Vec<u8>,
    pub family: String,
    pub style: FontStyle,
}
impl Face {
    pub fn new(file: &Font) -> Self {
        let bytes = std::fs::read(&file.path).unwrap();
        Face {
            bytes,
            family: file.family.clone(),
            style: file.style,
        }
    }
}
