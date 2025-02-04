use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

use super::content::Content;
use super::index::Index;
use super::manifest::Manifest;
use super::metadata::Metadata;
use super::spine::Spine;
use super::{cow_to_string, EpubError, Item};

#[allow(dead_code)]
pub struct Book {
    sourcefile: ZipArchive<File>,
    contents_dir: PathBuf,
    metadata: Metadata,
    index: Index,
}
impl Book {
    pub fn new<P: AsRef<Path>>(path: &P) -> Result<Self, EpubError> {
        // open epub file
        let epub_file = File::open(path).unwrap();
        let mut epub = ZipArchive::new(epub_file).unwrap();

        // read META-INF
        let mut file_bytes = Vec::new();
        {
            let mut container = epub.by_name("META-INF/container.xml").unwrap();
            let _ = container.read_to_end(&mut file_bytes).unwrap();
        }
        let file_contents = std::str::from_utf8(&file_bytes).unwrap();
        let rootfile = find_rootfile(file_contents)?;
        let contents_dir = rootfile.parent().unwrap().to_owned();

        // parse the contents.opf
        let mut file_bytes = Vec::new();
        let mut contents_opf = epub
            .by_name(rootfile.to_str().unwrap())
            .map_err(|_| EpubError::File)?;
        let _ = contents_opf.read_to_end(&mut file_bytes).unwrap();
        let file_contents = std::str::from_utf8(&file_bytes).unwrap();
        let mut reader = Reader::from_str(file_contents);

        let mut metadata = None;
        let mut manifest = None;
        let mut spine = None;
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
                    _ => (),
                },

                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }

        drop(contents_opf);

        let metadata = metadata.ok_or(EpubError::XmlField("metadata".into()))?;
        let manifest = manifest.ok_or(EpubError::XmlField("manifest".into()))?;
        let spine = spine.ok_or(EpubError::XmlField("manifest".into()))?;
        let index = Index::new(&manifest.items, &spine.itemrefs);

        Ok(Book {
            sourcefile: epub,
            contents_dir,
            metadata,
            index,
        })
    }

    pub fn index(&self) -> &Index {
        &self.index
    }
    pub fn next(&self) -> Option<&Item> {
        None
    }
    // pub fn next_element(&mut self, id: &str) -> Result<Content, EpubError> {
    //     let item = self.index.next_item(id).unwrap();
    //     let item = item.to_owned();
    //     self.content(item)
    // }
    // pub fn prev_element(&mut self, id: &str) -> Result<Content, EpubError> {
    //     let item = self.index.prev_item(id).unwrap();
    //     let item = item.to_owned();
    //     self.content(item)
    // }
    pub fn content(&mut self, item: &Item) -> Result<Content, EpubError> {
        let full_path = self.contents_dir.join(item.href().path);
        let path_str = full_path.to_str().unwrap();
        read_document(&mut self.sourcefile, path_str).map(Content::new)
    }
}

fn find_rootfile(xml: &str) -> Result<PathBuf, EpubError> {
    let mut reader = Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"rootfile" => {
                if let Ok(Some(attr)) = e.try_get_attribute("full-path") {
                    return cow_to_string(attr.value).map(PathBuf::from);
                }
            }
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.error_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }
    }
    Err(EpubError::File)
}

fn read_document(sourcefile: &mut ZipArchive<File>, id: &str) -> Result<Vec<u8>, EpubError> {
    let mut file_bytes = Vec::new();
    let mut z = sourcefile.by_name(id)?;
    z.read_to_end(&mut file_bytes)
        .map(|_| file_bytes)
        .map_err(|e| EpubError::Zipfile(e.into()))
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
