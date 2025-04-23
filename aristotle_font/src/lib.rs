pub mod builder;
pub mod fonts;
pub mod geom;
pub mod renderer;

use ttf_parser::GlyphId;

use self::fonts::{Family, FontStyle};
use self::geom::{Point, Rect};

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

#[derive(Clone, Default, Debug)]
pub struct FontWeight(f32);

#[derive(Clone, Default, Debug)]
pub struct TextObject {
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
    pub size: Option<f32>,
    pub style: Option<FontStyle>,
    pub weight: Option<FontWeight>,
}
impl TypesetObject {
    pub fn new(glyphs: Vec<Glyph>, start: Point, caret: Point) -> Self {
        Self {
            start,
            glyphs,
            caret,
            ..Default::default()
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct Glyph {
    gid: GlyphId,
    pos: Point,
    dim: Rect,
}
