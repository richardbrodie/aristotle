use std::cell::RefCell;
use std::rc::Rc;

use crate::epub::Content as BookContent;
use crate::epub::ContentElement as EpubElement;
use crate::epub::TextElement;
use crate::epub::TextStyle;
use crate::font::fonts::Family;
use crate::font::fonts::FontStyle;
use crate::font::raster::raster;
use crate::font::typeset::Element;
use crate::font::typeset::TResult;
use crate::font::typeset::Text;
use crate::font::FontError;
use crate::font::TextObject;
use crate::font::Typesetter;

pub struct Content {
    typesetter: Rc<RefCell<Typesetter>>,
    book_content: Option<BookContent>,
    pages: Vec<Page>,
}
impl Content {
    pub fn new(t: Rc<RefCell<Typesetter>>) -> Self {
        Self {
            typesetter: t,
            book_content: None,
            pages: vec![],
        }
    }
    pub fn set_content(&mut self, content: BookContent) {
        self.pages.clear();
        self.book_content = Some(content);
        self.typeset().unwrap();
    }
    pub fn page(&self, idx: usize) -> Option<&Page> {
        self.pages.get(idx)
    }
    pub fn len(&self) -> usize {
        self.pages.len()
    }
    pub fn typeset(&mut self) -> Result<(), ()> {
        let mut p = Page::default();
        let t = self.typesetter.borrow();
        let mut c = t.new_caret();
        let Some(content) = self.book_content.as_ref() else {
            tracing::error!("attempted to typset with empty content");
            return Err(());
        };
        let t = self.typesetter.borrow();
        for elem in content.content().iter() {
            match elem {
                EpubElement::Linebreak => {
                    if p.text_elements.is_empty() {
                        continue;
                    }
                    match t.linebreak(c) {
                        Ok(point) => c = point,
                        Err(FontError::PageOverflow) => {
                            tracing::info!("l: filled page with element");
                            self.pages.push(p);
                            p = Page::default();
                            c = t.new_caret();
                        }
                        Err(e) => {
                            tracing::error!("{:?}", e);
                            panic!()
                        }
                    }
                }
                EpubElement::Paragraph => match t.paragraph(c) {
                    Ok(point) => c = point,
                    Err(FontError::PageOverflow) => {
                        tracing::info!("p: filled page with element");
                        self.pages.push(p);
                        p = Page::default();
                        c = t.new_caret();
                    }
                    Err(e) => {
                        tracing::error!("{:?}", e);
                        panic!()
                    }
                },
                EpubElement::Inline(i) => {
                    let to = convert(i);
                    match t.text(c, &to) {
                        TResult::Overflow {
                            processed,
                            remainder,
                        } => {
                            tracing::info!(
                                "i: element overflowed at char {}",
                                to.raw_text.len() - remainder.len()
                            );
                            p.text_elements.push(processed.text);
                            self.pages.push(p);
                            p = Page::default();
                            c = t.new_caret();
                            let to1 = TextObject {
                                raw_text: remainder.trim().to_owned(),
                                style: to.style,
                                ..Default::default()
                            };
                            let TResult::Ok(Element { text, caret }) = t.text(c, &to1) else {
                                tracing::error!("failed to split an overflowing element");
                                panic!();
                            };
                            c = caret;
                            p.text_elements.push(text);
                        }
                        TResult::Error(e) => {
                            tracing::error!("{:?}", e);
                            panic!()
                        }
                        TResult::Ok(Element { caret, text }) => {
                            c = caret;
                            p.text_elements.push(text)
                        }
                    }
                }
                EpubElement::Heading(i) => {
                    let to = convert(i);
                    match t.heading(c, &to) {
                        TResult::Overflow {
                            processed: _,
                            remainder,
                        } => {
                            tracing::info!(
                                "h: element overflowed at char {}",
                                to.raw_text.len() - remainder.len()
                            );
                        }
                        TResult::Error(e) => {
                            tracing::error!("{:?}", e);
                            panic!()
                        }
                        TResult::Ok(Element { caret, text }) => {
                            c = caret;
                            p.text_elements.push(text)
                        }
                    }
                }
            }
        }
        self.pages.push(p);
        Ok(())
    }
}

fn convert(i: &TextElement) -> TextObject {
    let s = match i.style {
        TextStyle::Italic => FontStyle::Italic,
        TextStyle::Bold => FontStyle::Bold,
        TextStyle::Regular => FontStyle::Regular,
    };
    TextObject {
        raw_text: i.content.clone(),
        style: Some(s),
        ..Default::default()
    }
}
fn split(to: &TextObject, idx: usize) -> (TextObject, TextObject) {
    let (left, right) = to.raw_text.split_at(idx);
    tracing::info!("last char: {:?}", left.get(left.len() - 3..));
    (
        TextObject {
            raw_text: left.trim().to_owned(),
            style: to.style,
            ..Default::default()
        },
        TextObject {
            raw_text: right.trim().to_owned(),
            style: to.style,
            ..Default::default()
        },
    )
}

#[derive(Debug, Default)]
pub struct Page {
    text_elements: Vec<Text>,
}
impl Page {
    pub fn raster<F>(&self, fam: &Family, mut f: F) -> Result<(), FontError>
    where
        F: FnMut(u32, u32, u8),
    {
        self.text_elements
            .iter()
            .try_for_each(|e| raster(fam, e, &mut f))
    }
}
