use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum FontError {
    #[error("")]
    MissingFace,
    #[error("")]
    TtfParse,
    #[error("")]
    Raster,
    #[error("")]
    PageOverflow,
    #[error("")]
    ContentOverflow(usize),
    #[error("")]
    NoGlyph(char),
}
