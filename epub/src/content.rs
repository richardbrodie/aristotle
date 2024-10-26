use std::borrow::Borrow;

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

    use crate::ContentElement;

    use super::paragraphs;

    const FULL_XML: &str = r#"
<?xml version='1.0' encoding='utf-8'?>
<!DOCTYPE html PUBLIC '-//W3C//DTD XHTML 1.1//EN' 'http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd'>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en"><head>
<link href="5550132427476565403_cover.jpg" rel="icon" type="image/x-cover" id="id-2700600724960216573"/>

<title>
  The Project Gutenberg eBook of Pride and prejudice, by Jane Austen.
</title>

  <link href="0.css" rel="stylesheet" type="text/css"/>
<link href="pgepub.css" rel="stylesheet" type="text/css"/>
<meta name="generator" content="Ebookmaker 0.12.47 by Project Gutenberg"/>
</head>
<body class="x-ebookmaker x-ebookmaker-2"><h2 id="pgepubid00169"><a id="CHAPTER_XX"/><span id="img_images_i_168_a.jpg"></span>
<br/>
<br/>
CHAPTER XX.</h2>

<p class="nind"><span class="letra">
<span id="img_images_i_168_b.png">M</span></span>R. COLLINS was not left long to the silent contemplation of his
successful love; for Mrs. Bennet, having dawdled about in the vestibule
to watch for the end of the conference, no sooner saw Elizabeth open the
door and with quick step pass her towards the staircase, than she
entered the breakfast-room, and congratulated both him and herself in
warm terms on the happy prospect of their nearer connection. Mr. Collins
received and returned these felicitations with equal pleasure, and then
proceeded to relate the particulars of their interview, with the result
of which he trusted he had every reason to be satisfied, since the
refusal which his cousin had steadfastly given him would naturally flow
from her bashful modesty and the genuine delicacy of her character.<span class="x-ebookmaker-pageno" title="{140}"><a id="page_140" title="{140}"></a></span></p>

<p>This information, however, startled Mrs. Bennet: she would have been
glad to be equally satisfied that her daughter had meant to encourage
him by protesting against his proposals, but she dared not believe it,
and could not help saying so.</p>

<p>“But depend upon it, Mr. Collins,” she added, “that Lizzy shall be
brought to reason. I will speak to her about it myself directly. She is
a very headstrong, foolish girl, and does not know her own interest; but
I will <i>make</i> her know it.”</p>
        "#;

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
