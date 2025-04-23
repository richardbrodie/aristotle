use crate::Error;

#[derive(Debug, Default)]
pub struct Spine {
    _toc: String,
    itemrefs: Vec<String>,
}
impl Spine {
    pub fn extract(doc: &roxmltree::Document<'_>) -> Result<Spine, Error> {
        let spine_node = doc.descendants().find(|n| n.has_tag_name("spine")).unwrap();

        let toc = spine_node
            .attributes()
            .find(|attr| attr.name() == "toc")
            .map(|a| a.value().to_owned())
            .unwrap();
        let itemrefs: Vec<_> = spine_node
            .children()
            .filter_map(|node| match node.has_tag_name("itemref") {
                true => node
                    .attributes()
                    .find(|a| a.name() == "idref")
                    .map(|a| a.value().to_owned()),
                false => None,
            })
            .collect();
        Ok(Spine {
            _toc: toc,
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
