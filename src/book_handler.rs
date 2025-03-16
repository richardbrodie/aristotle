use std::path::Path;

use crate::config::Config;
use crate::epub::{Book, Item};
use crate::font::fonts::Family;
use crate::font::{TypesetConfig, Typesetter};
use crate::paginate::{paginate, Page};

#[derive(Debug)]
pub enum Error {
    NoChapter,
}

pub struct BookHandler {
    book: Book,
    // config: TypesetConfig,
    typesetter: Typesetter,
    current_chapter: Item,
    current_page: usize,
    pages: Vec<Page>,
}
impl BookHandler {
    pub fn new<P: AsRef<Path>>(path: &P, ts: Typesetter) -> Self {
        let book = Book::new(path).unwrap();
        let index = book.index();
        for i in index.items() {
            println!("item: {:?}", i);
        }
        let first_content = book.index().first_item().unwrap().to_owned();
        Self {
            book,
            typesetter: ts,
            current_chapter: first_content,
            current_page: 0,
            pages: vec![],
        }
    }

    pub fn page(&self) -> Option<&Page> {
        self.pages.get(self.current_page)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.typesetter.resize(width, height);
    }

    pub fn next_page(&mut self) -> Result<(), Error> {
        let pages = self.pages.len();
        if pages > 0 && self.current_page < pages - 1 {
            tracing::info!(
                "next page: chapter: {}, new page: {} of {}",
                self.current_chapter.id(),
                self.current_page + 2,
                pages
            );
            self.current_page += 1;
        } else {
            // last page so get new content
            let Some(next) = self.book.index().next_item(&self.current_chapter) else {
                return Err(Error::NoChapter);
            };
            let next = next.to_owned();
            let Ok(next_chapter) = self.book.content(&next) else {
                return Err(Error::NoChapter);
            };

            let pages = paginate(&next_chapter, &self.typesetter);
            tracing::info!(
                "next chapter: cur: {}, new: {}, pages: {}",
                self.current_chapter.id(),
                next.id(),
                pages.len()
            );
            self.pages = pages;
            self.current_chapter = next;
            self.current_page = 0;
        }
        return Ok(());
    }
    pub fn prev_page(&mut self) -> Result<(), Error> {
        // let pages = self.pages.len();
        if self.current_page > 0 {
            self.current_page -= 1;
        } else {
            // last page so get new content
            let Some(prev) = self.book.index().prev_item(&self.current_chapter) else {
                return Err(Error::NoChapter);
            };
            let prev = prev.to_owned();
            let Ok(prev_chapter) = self.book.content(&prev) else {
                return Err(Error::NoChapter);
            };

            let pages = paginate(&prev_chapter, &self.typesetter);
            self.pages = pages;
            self.current_chapter = prev;
            self.current_page = self.pages.len() - 1;
        }
        return Ok(());
    }
}
