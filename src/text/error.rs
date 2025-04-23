#[derive(Clone, Debug)]
pub enum FontError {
    MissingFace,
    TtfParse,
    Raster,
    PageOverflow,
    ContentOverflow(usize),
    NoGlyph(char),
}
