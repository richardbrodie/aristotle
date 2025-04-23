use super::{error::ContentError, html::Node, index::IndexElement};

#[derive(Debug)]
pub struct Content {
    pub item: IndexElement,
    pub node: Node,
}
impl Content {
    pub fn new(elem: &IndexElement, input: &[u8]) -> Result<Self, ContentError> {
        let node = Node::new(input).unwrap();
        Ok(Self {
            item: elem.to_owned(),
            node,
        })
    }
    pub fn id(&self) -> &str {
        &self.item.id()
    }
    pub fn node(&self) -> &Node {
        &self.node
    }
    pub fn iter(&self) -> impl Iterator<Item = &Node> {
        self.node.iter()
    }
}
