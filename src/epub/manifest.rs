use quick_xml::{events::Event, Reader};

use super::error::ContentError;

#[derive(Debug, Default)]
pub struct Manifest {
    items: Vec<Item>,
}
impl Manifest {
    pub fn extract(reader: &mut Reader<&[u8]>) -> Result<Self, ContentError> {
        let mut depth = 1;
        let mut metadata = Self::default();
        loop {
            let mut item = Item::default();
            match reader.read_event() {
                Ok(Event::Empty(ref e)) if e.name().as_ref() == b"item" => {
                    for attr in e.attributes() {
                        let attr = attr?;
                        let val = attr.unescape_value()?.into_owned();
                        match attr.key.as_ref() {
                            b"href" => item.href = val,
                            b"id" => item.id = val,
                            b"media-type" => item.mediatype = val,
                            _ => (),
                        }
                    }
                    metadata.items.push(item);
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
        Ok(metadata)
    }
    pub fn item(&self, id: &str) -> Option<&Item> {
        self.items.iter().find(|i| i.id == id)
    }
}

#[derive(Debug, Default)]
pub struct Item {
    id: String,
    pub href: String,
    mediatype: String,
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
