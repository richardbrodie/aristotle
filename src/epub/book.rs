use std::{fs::File, io::Read, path::Path};

use quick_xml::{events::Event, Reader};
use zip::ZipArchive;

use crate::epub::spine::Spine;

use super::{
    content::Content,
    error::Error,
    index::{Index, IndexElement},
    manifest::Manifest,
    metadata::Metadata,
    zip::{find_rootfile, read_document},
    Indexable,
};

#[derive(Debug, Default)]
pub struct Book {
    source_zip: Option<ZipArchive<File>>,
    index: Index,
    metadata: Metadata,
    content_buffer: Vec<u8>,
}

impl Book {
    pub fn new<P>(path: &P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let epub_file = File::open(path)?;
        let mut epub = ZipArchive::new(epub_file)?;

        // read META-INF
        let mut file_bytes = Vec::new();
        {
            let mut file = epub.by_name("META-INF/container.xml")?;
            let _ = file.read_to_end(&mut file_bytes)?;
        }
        let file_contents = std::str::from_utf8(&file_bytes)?;

        // open the rootfile
        let rootfile_path = find_rootfile(file_contents)?;
        let contents_dir = rootfile_path
            .parent()
            .unwrap_or(Path::new("OEBPS"))
            .to_owned();

        // parse the rootfile contents
        let rootfile_path = rootfile_path.to_str().ok_or(Error::ZipFile)?;
        read_document(&mut epub, rootfile_path, &mut file_bytes)?;
        let rootfile_contents = std::str::from_utf8(&file_bytes)?;
        let mut reader = Reader::from_str(rootfile_contents);

        let mut book = Book::default();

        let mut manifest = Manifest::default();
        let mut spine = Spine::default();

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => match e.name().as_ref() {
                    b"metadata" => {
                        let metadata = Metadata::extract(&mut reader)?;
                        book.metadata = metadata;
                    }
                    b"manifest" => {
                        manifest = Manifest::extract(&mut reader)?;
                    }
                    b"spine" => {
                        spine = Spine::extract(&mut reader)?;
                    }
                    _ => (),
                },
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }

        book.index = Index::new(manifest, spine, contents_dir);
        book.source_zip = Some(epub);
        Ok(book)
    }
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    pub fn index(&self) -> impl Iterator<Item = &IndexElement> {
        self.index.iter()
    }
}

impl Indexable for Book {
    fn content(&mut self, id: &str) -> Result<Content, Error> {
        let item = self.index.element(id).ok_or(Error::ContentNotFound)?;
        let zip = self.source_zip.as_mut().ok_or(Error::ZipFile)?;
        read_document(zip, item.path(), &mut self.content_buffer)?;
        Content::new(id, &self.content_buffer).map_err(|e| e.into())
    }
    fn first(&mut self) -> Result<Content, Error> {
        let item = self.index.first().ok_or(Error::ContentNotFound)?;
        let zip = self.source_zip.as_mut().ok_or(Error::ZipFile)?;
        read_document(zip, item.path(), &mut self.content_buffer)?;
        Content::new(item.id(), &self.content_buffer).map_err(|e| e.into())
    }
    fn next(&mut self, cur: &str) -> Result<Content, Error> {
        let item = self.index.next(cur).ok_or(Error::ContentNotFound)?;
        let zip = self.source_zip.as_mut().ok_or(Error::ZipFile)?;
        read_document(zip, item.path(), &mut self.content_buffer)?;
        Content::new(item.id(), &self.content_buffer).map_err(|e| e.into())
    }
    fn prev(&mut self, cur: &str) -> Result<Content, Error> {
        let item = self.index.prev(cur).ok_or(Error::ContentNotFound)?;
        let zip = self.source_zip.as_mut().ok_or(Error::ZipFile)?;
        read_document(zip, item.path(), &mut self.content_buffer)?;
        Content::new(item.id(), &self.content_buffer).map_err(|e| e.into())
    }
}
