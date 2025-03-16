use std::{fs::File, io::Read, path::PathBuf};

use quick_xml::{events::Event, Reader};
use zip::ZipArchive;

use super::error::{ContentError, Error};

pub fn find_rootfile(xml: &str) -> Result<PathBuf, Error> {
    let mut reader = Reader::from_str(xml);
    let mut buf = vec![];
    loop {
        let a = reader
            .read_event_into(&mut buf)
            .map_err(|e| Error::Content(e.into()))?;
        match a {
            Event::Empty(ref e) if e.name().as_ref() == b"rootfile" => {
                let a = e.try_get_attribute("full-path");
                let Some(attr) = a.map_err(|e| Error::Content(e.into()))? else {
                    return Err(ContentError::InvalidXml.into());
                };
                let p = std::str::from_utf8(&attr.value)?;
                return Ok(PathBuf::from(p));
            }
            Event::Eof => break,
            _ => (),
        }
    }
    // the loop ended without an EOF event, that's not good
    Err(Error::ZipFile)
}

pub fn read_document(
    sourcefile: &mut ZipArchive<File>,
    id: &str,
    buf: &mut Vec<u8>,
) -> Result<(), Error> {
    buf.clear();
    let mut z = sourcefile.by_name(id)?;
    let _ = z.read_to_end(buf)?;
    Ok(())
}
