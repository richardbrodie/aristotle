pub mod fonts;
pub mod geom;
pub mod raster;
pub mod typeset;

use fonts::FontStyle;
use geom::{Point, Rect};
use ttf_parser::GlyphId;

pub use typeset::{TypesetElement, Typesetter};

#[derive(Clone, Debug)]
pub enum Error {
    MissingFace,
    Raster,
    PageOverflow,
    ContentOverflow(usize),
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TypesetConfig {
    pub point_size: f32,
    pub page_width: u32,
    pub page_height: u32,
    pub horizontal_margin: f32,
    pub vertical_margin: f32,
}

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
    dim: Rect,
}
