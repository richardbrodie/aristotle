use crate::font::FontError;

use super::{index::IndexedFont, style::FontStyle};

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

    pub fn scaled_height(&self, point_size: f32) -> Result<f32, FontError> {
        let scale = self.scale_factor(point_size);
        self.as_ttf_face().map(|face| scale * face.height() as f32)
    }
    pub fn space_width(&self, point_size: f32) -> Result<f32, FontError> {
        let scale = self.scale_factor(point_size);
        let face = self.as_ttf_face()?;
        let gid = face.glyph_index(' ').ok_or(FontError::TtfParse)?;
        face.glyph_hor_advance(gid)
            .ok_or(FontError::TtfParse)
            .map(|adv| adv as f32 * scale)
    }

    pub fn as_ttf_face(&self) -> Result<ttf_parser::Face, FontError> {
        ttf_parser::Face::parse(&self.bytes, 0).map_err(|_| FontError::TtfParse)
    }
}
