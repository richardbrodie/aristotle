use thiserror::Error;

use crate::text::TextError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("softbuffer failure")]
    Softbuffer(#[from] softbuffer::SoftBufferError),

    #[error("font")]
    Text(#[from] TextError),

    #[error("file")]
    File(#[from] std::io::Error),

    #[error("png")]
    Png(#[from] png::DecodingError),
}
