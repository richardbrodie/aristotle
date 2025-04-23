use std::borrow::Borrow;

use quick_xml::events::Event;
use quick_xml::Reader;

use super::{ContentElement, TextElement, TextStyle};

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
                    return paragraphs(&mut reader);
                }
                Err(_) => return vec![],
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }
        vec![]
    }
}

fn paragraphs(reader: &mut Reader<&[u8]>) -> Vec<ContentElement> {
    let mut elements: Vec<ContentElement> = vec![];
    let mut element = TextElement::default();
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"div" | b"p" => elements.push(ContentElement::Paragraph),
                b"tr" => elements.push(ContentElement::Linebreak),
                b"i" => element.style = TextStyle::Italic,
                b"b" | b"strong" => element.style = TextStyle::Bold,
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
                let t = t.unescape().unwrap();
                let tx: &str = t.borrow();
                let tx = tx.trim_matches('\n');
                if !tx.is_empty() {
                    element.content = tx
                        .chars()
                        .map(|mut c| {
                            if c == '\n' {
                                c = ' ';
                            }
                            c
                        })
                        .collect();
                    elements.push(ContentElement::Inline(element.clone()));
                };
            }
            Err(e) => tracing::error!("{:?}", e),
            t => tracing::info!("unmatched: {:?}", t),
        }
    }
    elements
}

#[cfg(test)]
mod tests {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    use super::ContentElement;

    use super::paragraphs;

    #[test]
    fn parse_paragraph() {
        let xml = r#"
<?xml version='1.0' encoding='utf-8'?>
<!DOCTYPE html PUBLIC '-//W3C//DTD XHTML 1.1//EN' 'http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd'>
<body class="x-ebookmaker x-ebookmaker-2">

<p class="nind"><span class="letra">
<span id="img_images_i_168_b.png">M</span></span>R. COLLINS was not left long to the silent contemplation of his
successful love;<span class="x-ebookmaker-pageno" title="{140}"><a id="page_140" title="{140}"></a></span></p>
</body>
        "#;
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"body" => {
                    let tokens = paragraphs(&mut reader);
                    assert!(matches!(tokens[0], ContentElement::Paragraph));

                    let ContentElement::Inline(left) = &tokens[1] else {
                        panic!();
                    };
                    assert_eq!(left.content, "M");

                    let ContentElement::Inline(left) = &tokens[2] else {
                        panic!();
                    };
                    assert_eq!(
                        left.content,
                        "R. COLLINS was not left long to the silent contemplation of his successful love;"
                    );
                }
                Err(_) => panic!(),
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }
    }
}
