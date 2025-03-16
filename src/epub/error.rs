#[derive(Debug, PartialEq)]
pub enum ContentError {
    InvalidXml,
    UnexpectedEof,
    Other(String),
}
impl From<quick_xml::Error> for ContentError {
    fn from(error: quick_xml::Error) -> Self {
        tracing::error!("xml error: {}", error);
        ContentError::InvalidXml
    }
}
impl From<quick_xml::events::attributes::AttrError> for ContentError {
    fn from(error: quick_xml::events::attributes::AttrError) -> Self {
        tracing::error!("xml attribute error: {}", error);
        Self::InvalidXml
    }
}

#[derive(Debug)]
pub enum Error {
    FileIO,
    ZipFile,
    String,
    ContentNotFound,
    Content(ContentError),
    Other(String),
}
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        tracing::error!("io error: {}", error);
        Self::FileIO
    }
}
impl From<::zip::result::ZipError> for Error {
    fn from(error: ::zip::result::ZipError) -> Self {
        tracing::error!("zip error: {}", error);
        Self::ZipFile
    }
}
impl From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Self {
        tracing::error!("string error: {}", error);
        Self::String
    }
}
impl From<ContentError> for Error {
    fn from(error: ContentError) -> Self {
        Self::Content(error)
    }
}
