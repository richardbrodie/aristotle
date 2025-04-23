use thiserror::Error;

use crate::text::FontError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("softbuffer failure")]
    Softbuffer(#[from] softbuffer::SoftBufferError),

    #[error("font")]
    Font(#[from] FontError),
}
