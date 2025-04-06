use crate::epub::{ElementVariant, Node};
use crate::font::caret::Caret;
use crate::font::fonts::{Family, FontStyle};
use crate::font::typeset::{TResult, TypesetText};
use crate::font::{raster, typeset, FontError, TypesetConfig};

enum BreakType {
    Line,
    Block,
}

#[derive(Debug, Default)]
pub struct Page {
    text_elements: Vec<TypesetText>,
}

impl Page {
    pub fn raster<F>(&self, fam: &Family, width: usize, mut f: F) -> Result<(), FontError>
    where
        F: FnMut(u32, usize),
    {
        self.text_elements
            .iter()
            .try_for_each(|e| raster::raster(fam, e, width, &mut f))
    }
}

pub fn paginate(content: &Node<'_>, config: &TypesetConfig) -> Vec<Page> {
    let mut pages = vec![];
    let mut p = Page::default();

    let mut text_type = FontStyle::Regular;
    let mut caret = Caret::new(config).unwrap();
    let mut newline = false;
    // let mut first_in_page = true;
    let mut break_type = None;

    for node in content.iter() {
        match node {
            Node::Element(elem) => match elem.variant() {
                ElementVariant::H1 | ElementVariant::H2 | ElementVariant::H3 => {
                    text_type = FontStyle::Bold;
                    break_type = Some(BreakType::Block);
                    // if !first_in_page {
                    //     newline = true;
                    // }
                    // first_in_page = false;
                    // caret.newline();
                }
                ElementVariant::Span => {
                    caret.space();
                }
                ElementVariant::P | ElementVariant::Div => {
                    text_type = FontStyle::Regular;
                    break_type = Some(BreakType::Block);
                    // if !first_in_page {
                    // newline = true;
                    // }
                    // first_in_page = false;
                    // caret.newline();
                    // caret.indent();
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
                            BreakType::Block => 1.5,
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

                let mut remaining = Some(text.to_string());
                while let Some(next) = remaining.take() {
                    let res = typeset::typeset(config, &mut caret, &next, text_type);
                    match res {
                        TResult::Ok(typeset_text) => {
                            p.text_elements.push(typeset_text);
                        }
                        TResult::Overflow {
                            processed,
                            remainder,
                        } => {
                            // commit the pre-overflow part
                            p.text_elements.push(processed);
                            pages.push(p);
                            // start a new page
                            p = Page::default();
                            caret.reset_location();
                            let overflowed_text = remainder.trim();
                            remaining = Some(overflowed_text.to_string());
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
                    // if !caret.overflows_vertically() {
                    //     // draw a horizontal line
                    //     // ⎯
                    // }
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
