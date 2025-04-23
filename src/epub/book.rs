use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

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
};

#[derive(Debug, Default)]
pub struct Book {
    source_zip: Option<ZipArchive<File>>,
    index: Index,
    metadata: Metadata,
    contents_dir: PathBuf,
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

        book.index = Index::new(manifest, spine, &contents_dir);
        book.source_zip = Some(epub);
        book.contents_dir = contents_dir;
        Ok(book)
    }
    // pub fn index(&self) -> &Index {
    //     &self.index
    // }
    pub fn file(&mut self, href: &str) -> Result<&[u8], Error> {
        let mut path = PathBuf::from(href);
        let parent = path.parent().unwrap();
        if parent != self.contents_dir {
            path = self.contents_dir.join(&path);
        }
        let href = path.to_str().unwrap();
        let zip = self.source_zip.as_mut().ok_or(Error::ZipFile)?;
        read_document(zip, href, &mut self.content_buffer)?;
        Ok(&self.content_buffer)
    }

    // pub fn image(&mut self, path: &str) -> Result<(), Error> {
    //     let path = self.contents_dir.join(path);
    //     let path_str = path.to_str().unwrap();
    //     let data = self.file(path_str).unwrap();
    //     Ok(())
    // }
    pub fn content(&mut self, elem: &IndexElement) -> Result<Content, Error> {
        let data = self.file(elem.path()).unwrap();
        Content::new(elem, &data).map_err(|e| e.into())
    }

    pub fn first(&mut self) -> Result<Content, Error> {
        let item = self.index.first().ok_or(Error::ContentNotFound)?;
        self.content(&item)
    }
    pub fn next(&mut self, cur: &str) -> Result<Content, Error> {
        let item = self.index.next(cur).ok_or(Error::ContentNotFound)?;
        self.content(&item)
    }
    pub fn prev(&mut self, cur: &str) -> Result<Content, Error> {
        let item = self.index.prev(cur).ok_or(Error::ContentNotFound)?;
        self.content(&item)
    }
}
