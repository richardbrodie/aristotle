use quick_xml::events::Event;
use quick_xml::Reader;

use crate::{text_to_string, Error};

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Metadata {
    title: String,
    language: String,
    identifier: String,
    author: Option<String>,
    published: Option<String>,
}
impl Metadata {
    pub fn extract(reader: &mut Reader<&[u8]>) -> Result<Self, Error> {
        let mut title = None;
        let mut language = None;
        let mut identifier = None;
        let mut author = None;
        let mut published = None;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => match e.name().local_name().as_ref() {
                    b"title" => {
                        title = Some(text_to_string(reader.read_text(e.name()))?);
                    }
                    b"language" => language = Some(text_to_string(reader.read_text(e.name()))?),
                    b"identifier" => identifier = Some(text_to_string(reader.read_text(e.name()))?),
                    b"creator" => {
                        author = reader
                            .read_text(e.name())
                            .map(|c| Some(c.into_owned()))
                            .map_err(|_| Error::StringParse)?;
                    }
                    b"published" => {
                        published = reader
                            .read_text(e.name())
                            .map(|c| Some(c.into_owned()))
                            .map_err(|_| Error::StringParse)?;
                    }
                    _ => (),
                },
                Ok(Event::End(ref e)) if e.name().as_ref() == b"metadata" => break,
                Err(e) => return Err(e.into()),
                _ => (),
            }
        }
        Ok(Self {
            title: title.ok_or(Error::XmlField("title".into()))?,
            language: language.ok_or(Error::XmlField("language".into()))?,
            identifier: identifier.ok_or(Error::XmlField("identifier".into()))?,
            author,
            published,
        })
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
                    assert_eq!(result.title, "Pride and Prejudice");
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
                    let result = Metadata::extract(&mut reader);
                    assert!(result.is_err());
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
