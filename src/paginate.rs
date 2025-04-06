use crate::epub::Content as BookContent;
use crate::epub::ContentElement as EpubElement;
// use crate::epub::TextElement;
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

pub fn paginate(content: &BookContent, ts: &Typesetter) -> Vec<Page> {
    let mut pages = vec![];
    let mut p = Page::default();
    // let c = TypesetConfig::default();
    // let t = Typesetter::new(&c).unwrap();
    let mut c = ts.new_caret();
    for elem in content.content().iter() {
        match elem {
            EpubElement::Linebreak => {
                if p.text_elements.is_empty() {
                    continue;
                }
                match ts.linebreak(c) {
                    Ok(point) => c = point,
                    Err(FontError::PageOverflow) => {
                        tracing::info!("l: filled page with element");
                        pages.push(p);
                        p = Page::default();
                        c = ts.new_caret();
                    }
                    Err(e) => {
                        tracing::error!("{:?}", e);
                        panic!()
                    }
                }
            }
            EpubElement::Paragraph => match ts.paragraph(c) {
                Ok(point) => c = point,
                Err(FontError::PageOverflow) => {
                    tracing::info!("p: filled page with element");
                    pages.push(p);
                    p = Page::default();
                    c = ts.new_caret();
                }
                Err(e) => {
                    tracing::error!("{:?}", e);
                    panic!()
                }
            },
            EpubElement::Inline(i) => {
                let to = convert(i);
                match ts.text(c, &to) {
                    TResult::Overflow {
                        processed,
                        remainder,
                    } => {
                        tracing::info!(
                            "i: element overflowed at char {}",
                            to.raw_text.len() - remainder.len()
                        );
                        p.text_elements.push(processed.text);
                        pages.push(p);
                        p = Page::default();
                        c = ts.new_caret();
                        let to1 = TextObject {
                            raw_text: remainder.trim().to_owned(),
                            style: to.style,
                            ..Default::default()
                        };
                        let TResult::Ok(Element { text, caret }) = ts.text(c, &to1) else {
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
                match ts.heading(c, &to) {
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
    pages.push(p); // append last page from last iteration
    pages
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
    pub fn raster<F>(&self, fam: &Family, width: usize, mut f: F) -> Result<(), FontError>
    where
        F: FnMut(u32, usize),
    {
        self.text_elements
            .iter()
            .try_for_each(|e| raster(fam, e, width, &mut f))
    }
}

#[cfg(test)]
mod tests {
    use crate::font::fonts::FontStyle;
    use crate::font::raster::test_family;
    use crate::font::{TypesetConfig, Typesetter};

    #[test]
    fn new_page_then_linebreak() {
        // given
        let f = test_family(FontStyle::Regular);
        let mut cfg = TypesetConfig::default();
        cfg.family = f;
        let ts = Typesetter::new(&cfg).unwrap();
        let caret = ts.new_caret();

        // when
        let new_caret = ts.linebreak(caret).unwrap();

        // then
        let goal = caret.add_y(ts.line_height());
        assert_eq!(goal, new_caret);
    }
}
