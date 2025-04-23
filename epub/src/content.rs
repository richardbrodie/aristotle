use quick_xml::events::Event;
use quick_xml::Reader;

use crate::{ContentElement, TextElement, TextStyle};

#[derive(Debug, Default)]
pub struct Content {
    raw_content: Vec<u8>,
}
impl Content {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { raw_content: bytes }
    }
    pub fn content(&self) -> Vec<ContentElement> {
        let str_content = std::str::from_utf8(&self.raw_content).unwrap();
        let mut reader = Reader::from_str(str_content);
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"body" => {
                    tracing::info!("start: {:?}", e);
                    return paragraphs(&mut reader);
                }
                Err(_) => return vec![],
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                u => tracing::info!("unknown: {:?}", u),
            }
        }
        return vec![];
    }
}

fn paragraphs(reader: &mut Reader<&[u8]>) -> Vec<ContentElement> {
    let mut elements: Vec<ContentElement> = vec![];
    let mut element = TextElement::default();
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"div" | b"p" => {
                    elements.push(ContentElement::Paragraph);
                }
                b"i" => {
                    element.style = TextStyle::Italic;
                }
                b"b" => {
                    element.style = TextStyle::Bold;
                }
                _ => (),
            },
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"body" => break,
                _ => element.style = TextStyle::Regular,
            },
            Ok(Event::Empty(ref e)) => match e.name().as_ref() {
                b"br" => elements.push(ContentElement::Linebreak),
                _ => element.style = TextStyle::Regular,
            },
            Ok(Event::Text(ref t)) => {
                let tx = t.unescape().unwrap().into_owned();
                let el = if &tx == "\n" {
                    ContentElement::Linebreak
                } else {
                    element.content = tx;
                    ContentElement::Inline(element.clone())
                };
                elements.push(el);
            }
            Err(e) => tracing::error!("{:?}", e),
            t => tracing::info!("unmatched: {:?}", t),
        }
    }
    return elements;
}
