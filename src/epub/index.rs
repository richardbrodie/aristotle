use super::element::MediaType;
use super::manifest::ManifestItem;

pub struct Index {
    items: Vec<Item>,
}
impl Index {
    pub fn new(manifest: &[ManifestItem], spine: &[String]) -> Self {
        let items: Vec<_> = spine
            .iter()
            .filter_map(|s| manifest.iter().find(|m| m.id() == s))
            .filter_map(|m| match m.mediatype() {
                _mt @ (MediaType::Image | MediaType::Xhtml) => Some(Item {
                    id: m.id().to_owned(),
                    href: m.href().to_owned(),
                    // media_type: mt,
                    // property: Property::Unknown,
                }),
                _ => None,
            })
            .collect();

        Self { items }
    }
    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.items.iter()
    }
    pub fn first_item(&self) -> Option<Item> {
        self.items().next().map(|i| i.to_owned())
    }
    pub fn next_item(&self, cur: &Item) -> Option<&Item> {
        self.items().skip_while(|i| i.id != cur.id).next()
    }
    pub fn prev_item(&self, cur: &Item) -> Option<&Item> {
        let mut e = None;
        for n in self.items() {
            if n.id == cur.id {
                return e;
            }
            e = Some(n);
        }
        None
    }
}

#[derive(Clone, Debug)]
pub struct ContentLocation {
    pub path: String,
    pub idref: Option<String>,
}
#[derive(Clone, Debug, Default)]
pub struct Item {
    id: String,
    href: String,
    // media_type: MediaType,
    // property: Property,
}
impl Item {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub(super) fn href(&self) -> ContentLocation {
        let mut components = self.href.split('#');
        let path = components.next().unwrap();
        let idref = components.next().map(ToOwned::to_owned);
        ContentLocation {
            path: path.to_owned(),
            idref,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum Property {
    CoverImage,
    Nav,
    Svg,
    #[default]
    Unknown,
}
impl From<&str> for Property {
    fn from(value: &str) -> Self {
        match value {
            "cover-image" => Property::CoverImage,
            "nav" => Property::Nav,
            "svg" => Property::Svg,
            _ => Property::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {

    use quick_xml::events::Event;
    use quick_xml::Reader;

    use crate::epub::manifest::Manifest;
    use crate::epub::spine::Spine;

    use super::Index;

    #[test]
    fn happy_path() {
        let xml = r#"
<?xml version='1.0' encoding='utf-8'?>
  <manifest>
    <item id="cover" href="cover.jpeg" media-type="image/jpeg"/>
    <item id="titlepage" href="titlepage.xhtml" media-type="application/xhtml+xml"/>
    <item id="id5" href="text/part0000.html" media-type="application/xhtml+xml"/>
    <item id="id6" href="text/part0001.html" media-type="application/xhtml+xml"/>
    <item id="id7" href="text/part0002.html" media-type="application/xhtml+xml"/>
  </manifest>
  <spine toc="ncx">
    <itemref idref="titlepage"/>
    <itemref idref="id5"/>
    <itemref idref="id6"/>
    <itemref idref="id7"/>
  </spine>
        "#;
        let mut reader = Reader::from_str(xml);
        let mut manifest = None;
        let mut spine = None;
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"manifest" => {
                    manifest = Some(Manifest::extract(&mut reader));
                }
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"spine" => {
                    spine = Some(Spine::extract(e, &mut reader));
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }
        let manifest = manifest.unwrap().unwrap();
        let spine = spine.unwrap().unwrap();
        let index = Index::new(&manifest.items, spine.itemrefs.as_slice());
        assert_eq!(index.items().count(), 4);
        assert_eq!(index.items[0].id, "titlepage");
        assert_eq!(index.items[1].href, "text/part0000.html");
        // assert_eq!(index.items[2].media_type, MediaType::Xhtml);
    }
}
