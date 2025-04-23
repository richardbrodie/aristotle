use super::{html::Node, index::IndexElement, EpubError};

#[derive(Debug)]
pub struct Content {
    pub item: IndexElement,
    pub node: Node,
}
impl Content {
    pub fn new(elem: &IndexElement, input: &[u8]) -> Result<Self, EpubError> {
        let node = Node::new(input)?;
        Ok(Self {
            item: elem.to_owned(),
            node,
        })
    }
    pub fn node(&self) -> &Node {
        &self.node
    }
    pub fn iter(&self) -> impl Iterator<Item = &Node> {
        self.node.iter()
    }
}
