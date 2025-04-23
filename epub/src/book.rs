use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

use crate::element::Element;
use crate::guide::Guide;
use crate::spine::Spine;
use crate::{content::Content, cow_to_string, Error};
use crate::{manifest::Manifest, metadata::Metadata};

#[allow(dead_code)]
pub struct Book {
    sourcefile: ZipArchive<File>,
    contents_dir: PathBuf,
    metadata: Metadata,
    manifest: Manifest,
    spine: Spine,
    guide: Option<Guide>,
}
impl Book {
    pub fn new(p: &Path) -> Result<Self, Error> {
        // open epub file
        let epub_file = File::open(p).unwrap();
        let mut epub = ZipArchive::new(epub_file).unwrap();

        // read META-INF
        let mut file_bytes = Vec::new();
        {
            let mut container = epub.by_name("META-INF/container.xml").unwrap();
            let _ = container.read_to_end(&mut file_bytes).unwrap();
        }
        let file_contents = std::str::from_utf8(&file_bytes).unwrap();
        let rootfile = find_rootfile(&file_contents)?;
        let contents_dir = rootfile.parent().unwrap().to_owned();

        // parse the contents.opf
        let mut file_bytes = Vec::new();
        let mut contents_opf = epub
            .by_name(rootfile.to_str().unwrap())
            .map_err(|_| Error::Zip)?;
        let _ = contents_opf.read_to_end(&mut file_bytes).unwrap();
        let file_contents = std::str::from_utf8(&file_bytes).unwrap();
        let mut reader = Reader::from_str(file_contents);

        let mut metadata = None;
        let mut manifest = None;
        let mut spine = None;
        let mut guide = None;
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"metadata" => {
                        metadata = Some(Metadata::extract(&mut reader)?);
                    }
                    b"manifest" => {
                        manifest = Some(Manifest::extract(&mut reader)?);
                    }
                    b"spine" => {
                        spine = Some(Spine::extract(e, &mut reader)?);
                    }
                    b"guide" => {
                        guide = Guide::extract(&mut reader)?;
                    }
                    _ => (),
                },

                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }

        drop(contents_opf);

        Ok(Book {
            sourcefile: epub,
            contents_dir,
            metadata: metadata.ok_or(Error::XmlField("metadata".into()))?,
            manifest: manifest.ok_or(Error::XmlField("manifest".into()))?,
            spine: spine.ok_or(Error::XmlField("spine".into()))?,
            guide,
        })
    }
    fn read_document(&mut self, id: &str) -> Result<Vec<u8>, Error> {
        let mut file_bytes = Vec::new();
        let mut z = self.sourcefile.by_name(id).map_err(|_| Error::Zip)?;
        if z.read_to_end(&mut file_bytes).is_ok() {
            return Ok(file_bytes);
        }
        Err(Error::Zip)
    }
    pub fn items(&self) -> impl Iterator<Item = Element> + '_ {
        self.spine
            .items()
            .map_while(move |id| self.manifest.find(id))
            .map(Element::new)
    }
    pub fn element(&self, id: &str) -> Option<Element> {
        self.manifest.find(id).map(Element::new)
    }
    pub fn next_item(&self, id: &str) -> Option<Element> {
        self.spine
            .next(id)
            .and_then(|i| self.manifest.find(i))
            .map(Element::new)
    }
    pub fn content(&mut self, id: &str) -> Option<Content> {
        let item = self.manifest.find(id).unwrap();
        let path = item.href().to_str().unwrap().to_owned();
        if let Ok(bytes) = self.read_document(&path) {
            return Some(Content::new(bytes));
        }
        None
    }
}

fn find_rootfile(xml: &str) -> Result<PathBuf, Error> {
    let mut reader = Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"rootfile" => {
                if let Ok(attr) = e.try_get_attribute("full-path") {
                    if let Some(v) = attr {
                        return cow_to_string(v.value).map(PathBuf::from);
                    }
                }
            }
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }
    }
    Err(Error::File)
}

#[cfg(test)]
mod tests {
    use super::find_rootfile;

    #[test]
    fn get_rootfile_path() {
        let xml = r#"
        <?xml version="1.0"?>
        <container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
           <rootfiles>
              <rootfile full-path="content.opf" media-type="application/oebps-package+xml"/>

           </rootfiles>
        </container>
        "#;
        let res = find_rootfile(xml);
        assert_eq!(res.unwrap().to_str().unwrap(), "content.opf");
    }
}
