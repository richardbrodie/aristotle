use std::fs::File;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

use crate::element::Element;
use crate::guide::Guide;
use crate::spine::Spine;
use crate::{manifest::Manifest, metadata::Metadata};
use crate::{Content, Error};

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
        let rootfile = find_rootfile(&mut epub)?;
        let contents_dir = rootfile.parent().unwrap().to_owned();

        // parse the contents.opf
        let mut file_bytes = Vec::new();
        let mut contents_opf = epub
            .by_name(rootfile.to_str().unwrap())
            .map_err(|_| Error::Zip)?;
        let _ = contents_opf.read_to_end(&mut file_bytes).unwrap();
        let file_contents = std::str::from_utf8(&file_bytes).unwrap();
        let doc = roxmltree::Document::parse(file_contents).unwrap();

        let metadata = Metadata::extract(&doc)?;
        let manifest = Manifest::extract(&doc)?;
        let spine = Spine::extract(&doc)?;
        let guide = Guide::extract(&doc)?;
        drop(contents_opf);

        Ok(Book {
            sourcefile: epub,
            contents_dir,
            metadata,
            manifest,
            spine,
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
        if let Some(item) = self.manifest.find(id) {
            let el = Element::new(item);
            if let Ok(bytes) = self.read_document(el.path().unwrap()) {
                return Some(Content::new(bytes));
            }
        }
        None
    }
}

fn find_rootfile<R>(epub_file: &mut ZipArchive<R>) -> Result<PathBuf, Error>
where
    R: Read + Seek,
{
    let mut file_bytes = Vec::new();
    {
        let mut container = epub_file.by_name("META-INF/container.xml").unwrap();
        let _ = container.read_to_end(&mut file_bytes).unwrap();
    }
    let file_contents = std::str::from_utf8(&file_bytes).unwrap();

    let doc = roxmltree::Document::parse(file_contents).unwrap();
    let rootfile_path = doc
        .descendants()
        .find(|n| n.has_tag_name("rootfile"))
        .and_then(|elem| {
            elem.attributes()
                .find(|n| n.name() == "full-path")
                .map(|a| a.value())
        })
        .map(PathBuf::from)
        .ok_or(Error::XmlDocument);
    rootfile_path
}
