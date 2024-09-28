mod builder;
mod fonts;
mod geom;
mod renderer;

use ttf_parser::GlyphId;

#[derive(Clone, Debug)]
pub enum Error {
    Typeset,
}

#[derive(Debug)]
pub struct RenderingConfig {
    pub point_size: f32,
    pub width: u32,
    pub height: u32,
    pub font: Option<Family>,
}

#[derive(Clone, Default)]
pub struct FontWeight(f32);

#[derive(Clone, Default)]
pub struct TextObject {
    pub start_pos: Point,
    pub end_pos: Option<Point>,
    pub raw_text: String,
    pub size: Option<f32>,
    pub style: Option<FontStyle>,
    pub weight: Option<FontWeight>,
}

#[derive(Clone, Default, Debug)]
pub struct TypesetObject {
    pub start: Point,
    pub caret: Point,
    pub glyphs: Vec<Glyph>,
}
impl TypesetObject {
    pub fn new(glyphs: Vec<Glyph>, start: Point, caret: Point) -> Self {
        Self {
            start,
            glyphs,
            caret,
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Glyph {
    gid: GlyphId,
    pos: Point,
    dim: Rect,
}

pub use fonts::{FontIndexer, Indexer};
pub use geom::{Point, Rect};
pub use renderer::TextRenderer;

use self::fonts::{Family, FontStyle};
