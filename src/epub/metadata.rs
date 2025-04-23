use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Deserialize;

use super::error::ContentError;

#[derive(Debug, Default, Deserialize)]
pub struct Metadata {
    title: Option<String>,
    language: Option<String>,
    author: Option<String>,
    identifier: Option<String>,
    published: Option<String>,
}

impl Metadata {
    pub fn extract(reader: &mut Reader<&[u8]>) -> Result<Self, ContentError> {
        let mut depth = 1;
        let mut metadata = Self::default();
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    depth += 1;
                    let key = e.name().local_name();
                    if let Ok(Event::Text(text)) = reader.read_event() {
                        let value = std::str::from_utf8(&text).map(ToOwned::to_owned).ok();
                        match key.as_ref() {
                            b"title" => metadata.title = value,
                            b"language" => metadata.language = value,
                            b"identifier" => metadata.identifier = value,
                            b"creator" => metadata.author = value,
                            b"published" => metadata.published = value,
                            _ => (),
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
        Ok(metadata)
    }
}

#[cfg(test)]
mod tests {

    use quick_xml::events::Event;
    use quick_xml::Reader;

    use super::Metadata;

    #[test]
    fn happy_path() {
        let xml = r#"
<?xml version='1.0' encoding='utf-8'?>
  <metadata xmlns:opf='http://www.idpf.org/2007/opf' xmlns:dc='http://purl.org/dc/elements/1.1/'>
    <dc:title>Pride and Prejudice</dc:title>
    <dc:language>en</dc:language>
    <dc:identifier id='BookId' opf:scheme='ISBN'>123456789X</dc:identifier>
    <dc:creator opf:file-as='Austen, Jane' opf:role='aut'>Jane Austen</dc:creator>
  </metadata>
        "#;
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"metadata" => {
                    let result = Metadata::extract(&mut reader).unwrap();
                    assert_eq!(result.title, Some("Pride and Prejudice".to_owned()));
                    assert_eq!(result.author, Some("Jane Austen".to_owned()));
                    assert!(result.published.is_none());
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }
    }

    #[test]
    fn missing_title() {
        let xml = r#"
        <?xml version='1.0' encoding='utf-8'?>
        <metadata xmlns:opf='http://www.idpf.org/2007/opf' xmlns:dc='http://purl.org/dc/elements/1.1/'>
          <dc:language>en</dc:language>
          <dc:identifier id='BookId' opf:scheme='ISBN'>123456789X</dc:identifier>
          <dc:creator opf:file-as='Austen, Jane' opf:role='aut'>Jane Austen</dc:creator>
        </metadata>
        "#;
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"metadata" => {
                    let result = Metadata::extract(&mut reader).unwrap();
                    assert!(result.title.is_none());
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }
    }

    #[test]
    fn no_metadata_node() {
        let xml = r#"
    <?xml version='1.0' encoding='utf-8'?>
    <foo>
        <bar></bar>
    </foo>
        "#;
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"metadata" => {
                    let result = Metadata::extract(&mut reader);
                    assert!(result.is_err());
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }
    }
}
