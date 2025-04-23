use crate::draw::{Canvas, Error, Image};
use crate::epub::{Book, ElementVariant, Node};
use crate::text::caret::Caret;
use crate::text::fonts::{Family, FontStyle};
use crate::text::geom::Point;
use crate::text::typeset::{TResult, TypesetText};
use crate::text::{typeset, TypesetConfig};

enum BreakType {
    Line,
    Block,
}

#[derive(Debug)]
enum PageElement {
    Text(TypesetText),
    Hr { start: Point, end: Point },
    Image(Point, crate::draw::Image),
}

#[derive(Debug, Default)]
pub struct Page {
    text_elements: Vec<PageElement>,
}

impl Page {
    pub fn raster(&self, fam: &Family, canvas: &mut Canvas) -> Result<(), Error> {
        for e in &self.text_elements {
            match e {
                PageElement::Text(t) => canvas.text(fam, t)?,
                PageElement::Hr { start, end } => canvas.draw_line(start, end)?,
                PageElement::Image(point, image) => canvas.image(point, image),
            }
        }
        Ok(())
    }
}

pub fn paginate(content: &Node, config: &TypesetConfig, book: &mut Book) -> Vec<Page> {
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
                ElementVariant::P | ElementVariant::Div | ElementVariant::Tr => {
                    text_type = FontStyle::Regular;
                    break_type = Some(BreakType::Block);
                }
                ElementVariant::B => {
                    text_type = FontStyle::Bold;
                }
                ElementVariant::I => {
                    text_type = FontStyle::Italic;
                }
                ElementVariant::Image => {
                    let attr = elem.attribute("xlink:href").unwrap();
                    let content = book.file(attr.value()).unwrap();
                    let image = Image::open(content).unwrap();

                    let img_height = config.page_height - 2 * config.vertical_margin as usize;
                    let scale = img_height as f32 / image.size.height as f32;
                    let small_image = image.rescale(scale);
                    let hoffset = (config.page_width
                        - (2 * config.horizontal_margin) as usize
                        - small_image.size.width)
                        / 2;

                    let point = caret.point().add_x(hoffset as f32);
                    p.text_elements.push(PageElement::Image(point, small_image));
                }
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
        }
    }

    // add the last non-overflowed page
    pages.push(p);
    pages
}
