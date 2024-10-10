#[derive(Debug, Default)]
pub struct Spine {
    toc: String,
    itemrefs: Vec<String>,
}
impl Spine {
    pub fn new(toc: String, itemrefs: Vec<String>) -> Self {
        Self { toc, itemrefs }
    }
    pub fn items(&self) -> impl Iterator<Item = &str> {
        self.itemrefs.iter().map(|i| i.as_ref())
    }
    pub fn next(&self, id: &str) -> Option<&str> {
        let iter = self.itemrefs.iter().skip_while(|i| *i != id);
        iter.skip(1).next().map(|s| s.as_ref())
    }
}
