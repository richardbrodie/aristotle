#[derive(Debug, Default)]
pub struct Spine {
    _toc: String,
    itemrefs: Vec<String>,
}
impl Spine {
    pub fn new(toc: String, itemrefs: Vec<String>) -> Self {
        Self {
            _toc: toc,
            itemrefs,
        }
    }
    pub fn items(&self) -> impl Iterator<Item = &str> {
        self.itemrefs.iter().map(|i| i.as_ref())
    }
    pub fn next(&self, id: &str) -> Option<&str> {
        let mut iter = self.itemrefs.iter().skip_while(|i| *i != id);
        iter.nth(1).map(|s| s.as_ref())
    }
}
