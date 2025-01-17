use crate::epub::ContentElement as EpubElement;
use crate::epub::TextStyle;
use crate::font::fonts::FontStyle;
use crate::font::ContentElement;
use crate::font::TextObject;

pub fn convert_content(elem: &EpubElement) -> ContentElement {
    match elem {
        EpubElement::Linebreak => ContentElement::Linebreak,
        EpubElement::Inline(i) => {
            let s = match i.style {
                TextStyle::Italic => FontStyle::Italic,
                TextStyle::Bold => FontStyle::Bold,
                TextStyle::Regular => FontStyle::Regular,
            };
            let to = TextObject {
                raw_text: i.content.clone(),
                style: Some(s),
                ..Default::default()
            };
            ContentElement::Text(to)
        }
        EpubElement::Paragraph => ContentElement::Paragraph,
        EpubElement::Heading(h) => ContentElement::Text(TextObject {
            raw_text: h.content.clone(),
            style: Some(FontStyle::Bold),
            ..Default::default()
        }),
    }
}
