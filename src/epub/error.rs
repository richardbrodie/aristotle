use thiserror::Error;

#[derive(Debug, Error)]
pub enum EpubError {
    #[error("file io")]
    FileIO(#[from] std::io::Error),

    #[error("zipfile")]
    ZipFile(#[from] zip::result::ZipError),

    #[error("string")]
    Utf8(#[from] std::str::Utf8Error),

    #[error("requested content not found: {0}")]
    ContentNotFound(String),

    #[error("xml")]
    Xml(#[from] quick_xml::Error),

    #[error("xml attribute")]
    XmlAttribute(#[from] quick_xml::events::attributes::AttrError),

    #[error("no epub rootfile")]
    Rootfile,

    #[error("xml file ended prematurely")]
    UnexpectedEof,
}
