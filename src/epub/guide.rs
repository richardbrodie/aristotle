use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use super::EpubError;

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Reference {
    ref_type: String,
    title: String,
    href: String,
}
impl Reference {
    pub fn parse(element: &BytesStart, reader: &mut Reader<&[u8]>) -> Result<Reference, EpubError> {
        let mut href = None;
        let mut ref_type = None;
        let mut title = None;

        for attr in element.attributes() {
            let attr = attr?;
            match attr.key.as_ref() {
                b"href" => {
                    href = Some(
                        attr.decode_and_unescape_value(reader.decoder())
                            .map(|t| t.into_owned())?,
                    )
                }
                b"type" => {
                    ref_type = Some(
                        attr.decode_and_unescape_value(reader.decoder())
                            .map(|t| t.into_owned())?,
                    )
                }
                b"title" => {
                    title = Some(
                        attr.decode_and_unescape_value(reader.decoder())
                            .map(|t| t.into_owned())?,
                    )
                }
                _ => (),
            }
        }

        Ok(Reference {
            ref_type: ref_type.ok_or(EpubError::XmlField("type".into()))?,
            title: title.ok_or(EpubError::XmlField("title".into()))?,
            href: href.ok_or(EpubError::XmlField("href".into()))?,
        })
    }
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Guide {
    references: Vec<Reference>,
}
impl Guide {
    pub fn extract(reader: &mut Reader<&[u8]>) -> Result<Option<Guide>, EpubError> {
        let mut references = Vec::new();
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    if e.name().as_ref() == b"reference" {
                        references.push(Reference::parse(e, reader)?);
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"guide" => break,
                Err(e) => return Err(e.into()),
                _ => (),
            }
        }
        Ok(Some(Guide { references }))
    }
}

#[cfg(test)]
mod tests {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    use super::Reference;

    #[test]
    fn reference_correct() {
        let xml = r#"
            <reference type='loi' title='List Of Illustrations' href='appendix.xhtml#figures' />
        "#;
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"reference" => {
                    let result = Reference::parse(e, &mut reader).unwrap();
                    assert_eq!(result.ref_type, "loi");
                    assert_eq!(result.title, "List Of Illustrations");
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }
    }

    #[test]
    fn reference_missing_field() {
        let xml = r#"
        <reference title='List Of Illustrations' href='appendix.xhtml#figures' />
        "#;
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"reference" => {
                    let result = Reference::parse(e, &mut reader);
                    assert!(result.is_err());
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }
    }
}
