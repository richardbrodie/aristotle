use quick_xml::{events::Event, Reader};

use super::error::ContentError;

#[derive(Debug, Default)]
pub struct Spine {
    pub items: Vec<String>,
}
impl Spine {
    pub fn extract(reader: &mut Reader<&[u8]>) -> Result<Self, ContentError> {
        let mut depth = 1;
        let mut spine = Self::default();
        loop {
            match reader.read_event() {
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"itemref" => {
                    for attr in e.attributes() {
                        let Ok(attr) = attr else {
                            continue;
                        };
                        if attr.key.as_ref() == b"idref" {
                            if let Ok(val) = attr.unescape_value() {
                                spine.items.push(val.into_owned());
                            }
                        }
                    }
                }
                Ok(Event::End(_)) => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                Ok(Event::Eof) => return Err(ContentError::UnexpectedEof),
                Err(e) => return Err(e.into()),
                _ => {}
            }
        }
        Ok(spine)
    }
}

#[cfg(test)]
mod tests {

    use quick_xml::events::Event;
    use quick_xml::Reader;

    use super::Spine;

    #[test]
    fn happy_path() {
        let xml = r#"
<?xml version='1.0' encoding='utf-8'?>
  <spine toc="ncx">
    <itemref idref="titlepage"/>
    <itemref idref="id5"/>
    <itemref idref="id6"/>
    <itemref idref="id7"/>
  </spine>
        "#;
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"spine" => {
                    let result = Spine::extract(&mut reader).unwrap();
                    assert_eq!(result.items.len(), 4);
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }
    }

    #[test]
    fn missing_itemref() {
        let xml = r#"
<?xml version='1.0' encoding='utf-8'?>
  <spine toc="ncx">
    <itemref idref="titlepage"/>
    <itemref idreef="id5"/>
    <itemref idref="id6"/>
    <itemref idref="id7"/>
  </spine>
        "#;
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"spine" => {
                    let result = Spine::extract(&mut reader).unwrap();
                    assert_eq!(result.items.len(), 3);
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }
    }
}
