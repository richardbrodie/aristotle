use std::{fs::File, io::Read, path::PathBuf};

use quick_xml::{events::Event, Reader};
use zip::ZipArchive;

use super::EpubError;

pub fn find_rootfile(xml: &str) -> Result<PathBuf, EpubError> {
    let mut reader = Reader::from_str(xml);
    let mut buf = vec![];
    loop {
        let a = reader.read_event_into(&mut buf)?;
        match a {
            Event::Empty(ref e) if e.name().as_ref() == b"rootfile" => {
                let Ok(Some(attr)) = e.try_get_attribute("full-path") else {
                    return Err(EpubError::Rootfile);
                };
                return std::str::from_utf8(&attr.value)
                    .map(PathBuf::from)
                    .map_err(Into::into);
            }
            Event::Eof => break,
            _ => (),
        }
    }
    // the loop ended without finding the rootfile path
    Err(EpubError::UnexpectedEof)
}

pub fn read_document(
    sourcefile: &mut ZipArchive<File>,
    id: &str,
    buf: &mut Vec<u8>,
) -> Result<(), EpubError> {
    buf.clear();
    let mut z = sourcefile.by_name(id)?;
    let _ = z.read_to_end(buf)?;
    Ok(())
}
