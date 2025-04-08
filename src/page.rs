use std::ops::DerefMut;

use crate::epub::{ElementVariant, Node};
use crate::font::caret::Caret;
use crate::font::fonts::{Family, FontStyle};
use crate::font::geom::Point;
use crate::font::typeset::{TResult, TypesetText};
use crate::font::{raster, typeset, FontError, TypesetConfig};

enum BreakType {
    Line,
    Block,
}

#[derive(Debug)]
enum PageElement {
    Text(TypesetText),
    Hr { start: Point, end: Point },
}

#[derive(Debug, Default)]
pub struct Page {
    text_elements: Vec<PageElement>,
}

impl Page {
    pub fn raster<B>(&self, fam: &Family, width: usize, buffer: &mut B) -> Result<(), FontError>
    where
        B: DerefMut<Target = [u32]>,
    {
        self.text_elements.iter().try_for_each(|e| match e {
            PageElement::Text(t) => raster::text(fam, t, width, buffer),
            PageElement::Hr { start, end } => raster::hr(start, end, width, buffer),
        })
    }
}

pub fn paginate(content: &Node<'_>, config: &TypesetConfig) -> Vec<Page> {
    let mut pages = vec![];
    let mut p = Page::default();

    let mut text_type = FontStyle::Regular;
    let mut caret = Caret::new(config).unwrap();
    let mut break_type = None;

    for node in content.iter() {
        match node {
            Node::Element(elem) => match elem.variant() {
                ElementVariant::H1 | ElementVariant::H2 | ElementVariant::H3 => {
                    text_type = FontStyle::Bold;
                    break_type = Some(BreakType::Block);
                }
                ElementVariant::Span => {
                    caret.space();
                }
                ElementVariant::P | ElementVariant::Div => {
                    text_type = FontStyle::Regular;
                    break_type = Some(BreakType::Block);
                }
                ElementVariant::B => {
                    text_type = FontStyle::Bold;
                }
                ElementVariant::I => {
                    text_type = FontStyle::Italic;
                }
                v => {
                    // tracing::info!("element: [{:?}]", v);
                }
            },
            Node::Text(text) => {
                if !p.text_elements.is_empty() {
                    if let Some(bt) = break_type.take() {
                        let lines = match bt {
                            BreakType::Block => 1.3,
                            BreakType::Line => 1.0,
                        };
                        if caret.overflows_vertically(lines) {
                            pages.push(p);
                            p = Page::default();
                            caret.reset_location();
                        } else {
                            caret.newline(lines);
                        }
                    }
                }

                let mut remaining = Some(text.chars().skip(0));
                while let Some(next) = remaining.take() {
                    let res = typeset::typeset(config, &mut caret, next, text_type);
                    match res {
                        TResult::Ok(typeset_text) => {
                            p.text_elements.push(PageElement::Text(typeset_text));
                        }
                        TResult::Overflow { processed, index } => {
                            // commit the pre-overflow part
                            p.text_elements.push(PageElement::Text(processed));
                            pages.push(p);

                            // start a new page
                            p = Page::default();
                            caret.reset_location();
                            let overflow = text.chars().skip(index);
                            remaining = Some(overflow);
                        }
                        TResult::Error(err) => {
                            tracing::error!("{:?}", err);
                            panic!()
                        }
                    }
                }

                text_type = FontStyle::Regular;
            }
            Node::Empty(tag) => match tag {
                ElementVariant::Br => {
                    break_type = Some(BreakType::Line);
                }
                ElementVariant::Hr => {
                    caret.newline(1.0);
                    let s = caret.point();
                    let e = config.page_width - config.horizontal_margin as usize;
                    let midline = caret.scaled_height() / 2.0;
                    let start = Point::new(s.x, (s.y + midline).floor());
                    let end = Point::new(e as f32, (s.y + midline).ceil());
                    p.text_elements.push(PageElement::Hr { start, end });
                }
                e => {
                    tracing::info!("empty: [{:?}]", e);
                }
            },
        }
    }

    // add the last non-overflowed page
    pages.push(p);
    pages
}
