use super::index::IndexedFont;
use super::FontStyle;

#[derive(Default, Clone, Debug)]
pub struct Face {
    bytes: Vec<u8>,
    _family: String,
    style: FontStyle,
    units_per_em: f32,
}
impl Face {
    pub fn new(file: &IndexedFont) -> Self {
        let f = ttf_parser::Face::parse(&file.bytes, 0).unwrap();
        let units_per_em = f.units_per_em() as f32;
        Face {
            bytes: file.bytes.clone(),
            _family: file.family.clone(),
            style: file.style,
            units_per_em,
        }
    }

    pub fn style(&self) -> FontStyle {
        self.style
    }

    pub fn scale_factor(&self, point_size: f32) -> f32 {
        let px_per_em = point_size * (96.0 / 72.0);
        px_per_em / self.units_per_em
    }

    pub fn as_face(&self) -> ttf_parser::Face {
        ttf_parser::Face::parse(&self.bytes, 0).unwrap()
    }
}
