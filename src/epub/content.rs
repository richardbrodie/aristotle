use super::{error::ContentError, html::Node};

#[derive(Debug, PartialEq)]
pub struct Content<'a> {
    id: String,
    node: Node<'a>,
}
impl<'a> Content<'a> {
    pub fn new(id: &str, input: &'a [u8]) -> Result<Self, ContentError> {
        let node = Node::parse(input).unwrap();
        Ok(Self {
            id: id.to_owned(),
            node,
        })
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn node(&self) -> &Node<'a> {
        &self.node
    }
    pub fn iter(&self) -> impl Iterator<Item = &Node<'_>> {
        self.node.iter()
    }
}
