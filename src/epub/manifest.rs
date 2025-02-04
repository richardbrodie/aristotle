use std::borrow::Cow;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use super::element::MediaType;
use super::EpubError;

#[derive(Debug, Default)]
pub struct Manifest {
    pub items: Vec<ManifestItem>,
}
impl Manifest {
    pub fn extract(reader: &mut Reader<&[u8]>) -> Result<Manifest, EpubError> {
        let mut items: Vec<ManifestItem> = Vec::new();
        loop {
            match reader.read_event() {
                Ok(Event::Empty(ref e)) => {
                    if e.name().as_ref() == b"item" {
                        items.push(ManifestItem::extract(reader, e)?);
                    }
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"manifest" => break,
                Err(e) => return Err(e.into()),
                _ => (),
            }
        }

        Ok(Manifest { items })
    }
    pub fn find(&self, id: &str) -> Option<&ManifestItem> {
        self.items.iter().find(|i| i.id == id)
    }
}

#[derive(Debug, Default)]
pub struct ManifestItem {
    id: String,
    href: String,
    mediatype: String,
}
impl ManifestItem {
    fn extract(reader: &mut Reader<&[u8]>, element: &BytesStart) -> Result<Self, EpubError> {
        let mut mediatype = Cow::Borrowed("");
        let mut href = Cow::Borrowed("");
        let mut id = Cow::Borrowed("");

        for attr_result in element.attributes() {
            let a = attr_result?;
            match a.key.as_ref() {
                b"href" => href = a.decode_and_unescape_value(reader.decoder())?,
                b"id" => id = a.decode_and_unescape_value(reader.decoder())?,
                b"media-type" => mediatype = a.decode_and_unescape_value(reader.decoder())?,
                _ => (),
            }
        }
        Ok(Self {
            id: id.into(),
            mediatype: mediatype.into(),
            href: href.into(),
        })
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn href(&self) -> &str {
        &self.href
    }
    pub fn mediatype(&self) -> MediaType {
        self.mediatype.as_str().into()
    }
}

#[cfg(test)]
mod tests {

    use quick_xml::events::Event;
    use quick_xml::Reader;

    use super::Manifest;

    #[test]
    fn happy_path() {
        let xml = r#"
<?xml version='1.0' encoding='utf-8'?>
  <manifest>
    <item id="cover" href="cover.jpeg" media-type="image/jpeg"/>
    <item id="titlepage" href="titlepage.xhtml" media-type="application/xhtml+xml"/>
    <item id="id5" href="text/part0000.html" media-type="application/xhtml+xml"/>
  </manifest>
        "#;
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"manifest" => {
                    let result = Manifest::extract(&mut reader).unwrap();
                    assert_eq!(result.items.len(), 3);
                    assert_eq!(result.items[0].id, "cover");
                    assert_eq!(result.items[1].href, "titlepage.xhtml");
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }
    }
}
