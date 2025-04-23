use thiserror::Error;

use super::typeset::TypesetText;

#[derive(Debug, Error)]
pub enum TextError {
    #[error("file io")]
    FileIO(#[from] std::io::Error),

    #[error("specified face not indexed")]
    MissingFace,

    #[error("unable to parse ttf file")]
    TtfParse(#[from] ttf_parser::FaceParsingError),

    #[error("")]
    PageOverflow,

    #[error("content overflowed")]
    ContentOverflow(TypesetText, usize),

    #[error("specified glyph {0} not found in font")]
    NoGlyph(char),
}
