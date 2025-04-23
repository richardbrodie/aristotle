use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use super::EpubError;

#[derive(Debug, Default)]
pub struct Spine {
    _toc: String,
    pub itemrefs: Vec<String>,
}
impl Spine {
    pub fn extract(element: &BytesStart, reader: &mut Reader<&[u8]>) -> Result<Spine, EpubError> {
        let toc = Some(find_attribute(element, reader, b"toc")?);
        let mut itemrefs = Vec::new();
        loop {
            match reader.read_event() {
                Ok(Event::Empty(ref e)) => {
                    if e.name().as_ref() == b"itemref" {
                        itemrefs.push(find_attribute(e, reader, b"idref")?);
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"spine" => break,
                Err(e) => return Err(e.into()),
                _ => (),
            }
        }

        Ok(Self {
            _toc: toc.ok_or(EpubError::XmlField("toc".into()))?,
            itemrefs,
        })
    }
    pub fn items(&self) -> impl Iterator<Item = &str> {
        self.itemrefs.iter().map(|i| i.as_ref())
    }
    pub fn next(&self, id: &str) -> Option<&str> {
        let mut iter = self.itemrefs.iter().skip_while(|i| *i != id);
        iter.nth(1).map(|s| s.as_ref())
    }
}

fn find_attribute(
    element: &BytesStart,
    reader: &mut Reader<&[u8]>,
    key: &[u8],
) -> Result<String, EpubError> {
    for attr in element.attributes() {
        let attr = attr?;
        if attr.key.as_ref() == key {
            return attr
                .decode_and_unescape_value(reader.decoder())
                .map(|t| t.into_owned())
                .map_err(Into::into);
        }
    }
    Err(EpubError::XmlAttribute)
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
                    let result = Spine::extract(e, &mut reader).unwrap();
                    assert_eq!(result._toc, "ncx");
                    assert_eq!(result.itemrefs.len(), 4);
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }
    }

    #[test]
    fn missing_toc() {
        let xml = r#"
<?xml version='1.0' encoding='utf-8'?>
  <spine tok="ncx">
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
                    let result = Spine::extract(e, &mut reader);
                    assert!(result.is_err());
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
                    let result = Spine::extract(e, &mut reader);
                    assert!(result.is_err());
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }
    }
}
