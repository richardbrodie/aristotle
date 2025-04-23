use std::path::Path;
use std::sync::{Arc, RwLock};

use crate::app::Error;
use crate::epub::{Book, Content, IndexElement};
use crate::page::{paginate, Page};
use crate::text::TypesetConfig;

pub struct BookHandler {
    book: Book,
    config: Arc<RwLock<TypesetConfig>>,
    current_chapter: Option<IndexElement>,
    current_page: usize,
    pages: Vec<Page>,
}
impl BookHandler {
    pub fn new<P: AsRef<Path>>(
        path: &P,
        config: Arc<RwLock<TypesetConfig>>,
    ) -> Result<Self, Error> {
        let book = Book::new(path)?;
        let mut b = Self {
            book,
            config,
            current_chapter: None,
            current_page: 0,
            pages: vec![],
        };

        {
            // go to first page
            let Content { item, node } = b.book.first()?;
            let c = b.config.read().map_err(|_| Error::RwLock)?;
            b.current_page = 0;
            b.current_chapter = Some(item);
            b.pages = paginate(&node, &c, &mut b.book)?;
        }

        Ok(b)
    }

    pub fn repaginate(&mut self) -> Result<(), Error> {
        if let Some(chap) = self.current_chapter.as_ref() {
            let content = self.book.content(chap)?;

            let c = self.config.read().map_err(|_| Error::RwLock)?;
            self.pages = paginate(content.node(), &c, &mut self.book)?;
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
            let Content { item, node } = match self.current_chapter.as_ref() {
                Some(elem) => self.book.next(elem.id()),
                None => self.book.first(),
            }?;

            let c = self.config.read().map_err(|_| Error::RwLock)?;
            let pages = paginate(&node, &c, &mut self.book)?;
            self.pages = pages;
            self.current_chapter = Some(item);
            self.current_page = 0;
        }
        Ok(())
    }

    pub fn prev_page(&mut self) -> Result<(), Error> {
        if self.current_page > 0 {
            self.current_page -= 1;
        } else {
            // first page so get new content
            let Content { item, node } = match self.current_chapter.as_ref() {
                Some(elem) => self.book.prev(elem.id()),
                None => self.book.first(),
            }?;
            let c = self.config.read().map_err(|_| Error::RwLock)?;
            let pages = paginate(&node, &c, &mut self.book)?;
            self.pages = pages;
            self.current_chapter = Some(item);
            self.current_page = self.pages.len() - 1;
        }
        Ok(())
    }
}
