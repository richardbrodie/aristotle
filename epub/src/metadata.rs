use crate::Error;

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
    pub fn extract(doc: &roxmltree::Document<'_>) -> Result<Self, Error> {
        let meta_node = doc
            .descendants()
            .find(|n| n.has_tag_name("metadata"))
            .ok_or(Error::XmlDocument)?;

        let mut title = None;
        let mut language = None;
        let mut identifier = None;
        let mut author = None;
        let mut published = None;
        for node in meta_node.children() {
            if node.tag_name().namespace() == Some("http://purl.org/dc/elements/1.1/") {
                match node.tag_name().name() {
                    "title" => title = node.text().map(ToOwned::to_owned),
                    "language" => language = node.text().map(ToOwned::to_owned),
                    "identifier" => identifier = node.text().map(ToOwned::to_owned),
                    "creator" => author = node.text().map(ToOwned::to_owned),
                    "date" => published = node.text().map(ToOwned::to_owned),
                    _ => {}
                }
            }
        }
        Ok(Self {
            title: title.ok_or(Error::XmlField)?,
            language: language.ok_or(Error::XmlField)?,
            identifier: identifier.ok_or(Error::XmlField)?,
            author,
            published,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Metadata;

    #[test]
    fn happy_path() {
        let xml = "<?xml version='1.0' encoding='utf-8'?>
  <metadata xmlns:opf='http://www.idpf.org/2007/opf' xmlns:dc='http://purl.org/dc/elements/1.1/'>
    <dc:title>Pride and Prejudice</dc:title>
    <dc:language>en</dc:language>
    <dc:identifier id='BookId' opf:scheme='ISBN'>123456789X</dc:identifier>
    <dc:creator opf:file-as='Austen, Jane' opf:role='aut'>Jane Austen</dc:creator>
  </metadata>";
        let doc = roxmltree::Document::parse(xml).unwrap();
        let result = Metadata::extract(&doc);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.title, "Pride and Prejudice");
        assert!(r.published.is_none());
    }

    #[test]
    fn missing_title() {
        let xml = "<?xml version='1.0' encoding='utf-8'?>
    <metadata xmlns:opf='http://www.idpf.org/2007/opf' xmlns:dc='http://purl.org/dc/elements/1.1/'>
      <dc:language>en</dc:language>
      <dc:identifier id='BookId' opf:scheme='ISBN'>123456789X</dc:identifier>
      <dc:creator opf:file-as='Austen, Jane' opf:role='aut'>Jane Austen</dc:creator>
    </metadata>";
        let doc = roxmltree::Document::parse(xml).unwrap();
        let result = Metadata::extract(&doc);
        assert!(result.is_err());
    }

    #[test]
    fn no_metadata_node() {
        let xml = "<?xml version='1.0' encoding='utf-8'?>
<foo>
<bar></bar>
</foo>";
        let doc = roxmltree::Document::parse(xml).unwrap();
        let result = Metadata::extract(&doc);
        assert!(result.is_err());
    }
}
