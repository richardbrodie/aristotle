use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::epub::{self, Book, Indexable};
use crate::font::TypesetConfig;
use crate::page::{paginate, Page};

#[derive(Debug)]
pub enum Error {
    NoChapter,
    Epub(epub::Error),
}
impl From<epub::Error> for Error {
    fn from(error: epub::Error) -> Self {
        Self::Epub(error)
    }
}

pub struct BookHandler {
    book: Book,
    config: Arc<RwLock<TypesetConfig>>,
    current_chapter: Option<String>,
    current_page: usize,
    pages: Vec<Page>,
}
impl BookHandler {
    pub fn new<P: AsRef<Path>>(path: &P, config: Arc<RwLock<TypesetConfig>>) -> Self {
        let book = Book::new(path).unwrap();

        Self {
            book,
            config,
            current_chapter: None,
            current_page: 0,
            pages: vec![],
        }
    }

    pub fn repaginate(&mut self) -> Result<(), Error> {
        if let Some(chap) = self.current_chapter.as_ref() {
            let content = self.book.content(chap)?;
            let node = content.iter().next().unwrap();

            let c = self.config.read().unwrap();
            self.pages = paginate(node, &c);
        }
        Ok(())
    }

    pub fn page(&self) -> Option<&Page> {
        self.pages.get(self.current_page)
    }

    pub fn next_page(&mut self) -> Result<(), Error> {
        let num_pages = self.pages.len();
        if num_pages > 0 && self.current_page < num_pages - 1 {
            self.current_page += 1;
        } else {
            // last page so get new content
            let content = match self.current_chapter.as_ref() {
                Some(id) => self.book.next(id),
                None => self.book.first(),
            }
            .map_err(|_| Error::NoChapter)?;

            let c = self.config.read().unwrap();
            let pages = paginate(content.node(), &c);
            self.pages = pages;
            self.current_chapter = Some(content.id().to_owned());
            self.current_page = 0;
        }
        Ok(())
    }

    pub fn prev_page(&mut self) -> Result<(), Error> {
        if self.current_page > 0 {
            self.current_page -= 1;
        } else {
            // first page so get new content
            let content = match self.current_chapter.as_ref() {
                Some(id) => self.book.prev(id),
                None => self.book.first(),
            }
            .map_err(|_| Error::NoChapter)?;
            let c = self.config.read().unwrap();
            let pages = paginate(content.node(), &c);
            self.pages = pages;
            self.current_chapter = Some(content.id().to_owned());
            self.current_page = self.pages.len() - 1;
        }
        Ok(())
    }
}
