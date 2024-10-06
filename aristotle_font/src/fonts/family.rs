use super::index::Font;

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

#[derive(Debug, Clone)]
pub struct Family {
    pub name: String,
    pub faces: Vec<Face>,
}

pub trait Faces {
    fn get_face(&self, style: FontStyle) -> Option<&Face>;
    fn styles(&self) -> impl Iterator<Item = FontStyle>;
}

impl Faces for Family {
    fn get_face(&self, style: FontStyle) -> Option<&Face> {
        self.faces.iter().find(|s| s.style == style)
    }
    fn styles(&self) -> impl Iterator<Item = FontStyle> {
        self.faces.iter().map(|f| f.style)
    }
}

#[derive(Default, Clone, Debug)]
pub struct Face {
    bytes: Vec<u8>,
    family: String,
    style: FontStyle,
    units_per_em: f32,
}
impl Face {
    pub fn new(file: &Font) -> Self {
        let bytes = std::fs::read(&file.path).unwrap();
        let f = ttf_parser::Face::parse(&bytes, 0).unwrap();
        let units_per_em = f.units_per_em() as f32;
        Face {
            bytes,
            family: file.family.clone(),
            style: file.style,
            units_per_em,
        }
    }

    pub fn scale_factor(&self, point_size: f32) -> f32 {
        let px_per_em = point_size * (96.0 / 72.0);
        px_per_em / self.units_per_em
    }

    pub fn as_face(&self) -> ttf_parser::Face {
        ttf_parser::Face::parse(&self.bytes, 0).unwrap()
    }
}
