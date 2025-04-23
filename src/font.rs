pub mod caret;
mod error;
pub mod fonts;
pub mod geom;
pub mod raster;
pub mod typeset;

use fonts::{Family, FontStyle};
use geom::{Point, Rect};
use ttf_parser::GlyphId;

pub use error::FontError;
// pub use typeset::Typesetter;

#[derive(Debug, Clone)]
pub struct TypesetConfig {
    pub family: Family,
    pub point_size: f32,
    pub page_width: usize,
    pub page_height: usize,
    pub horizontal_margin: u8,
    pub vertical_margin: u8,
}
impl Default for TypesetConfig {
    fn default() -> Self {
        Self {
            family: Family::default(),
            point_size: 18.0,
            page_width: 640,
            page_height: 480,
            horizontal_margin: 12,
            vertical_margin: 12,
        }
    }
}
// impl TypesetConfig {
//     pub fn with_family(mut self, f: Family) -> Self {
//         self.family = f;
//         self
//     }
// }

#[derive(Clone, Default, Debug)]
#[allow(dead_code)]
pub struct FontWeight(f32);

#[derive(Clone, Debug)]
pub enum ContentElement {
    Text(TextObject),
    Linebreak,
    Paragraph,
}

#[derive(Clone, Default, Debug)]
pub struct TextObject {
    pub raw_text: String,
    pub size: Option<f32>,
    pub style: Option<FontStyle>,
    pub weight: Option<FontWeight>,
}

#[derive(Clone, Default, Debug)]
pub struct Glyph {
    gid: GlyphId,
    pos: Point,
    pub dim: Rect,
}
