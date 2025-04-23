use std::str::FromStr;

use quick_xml::{
    events::{attributes::Attribute as QAttribute, BytesStart, Event},
    Reader,
};

use super::EpubError;

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub enum ElementVariant {
    A,
    B,
    Blockquote,
    Body,
    Br,
    Div,
    H1,
    H2,
    H3,
    I,
    Html,
    Hr,
    Tr,
    Image,
    P,
    Section,
    Span,
    #[default]
    Ignored,
}
impl FromStr for ElementVariant {
    type Err = EpubError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_ref() {
            "html" => Ok(Self::Html),
            "body" => Ok(Self::Body),
            "h1" => Ok(Self::H1),
            "h2" => Ok(Self::H2),
            "h3" => Ok(Self::H3),
            "p" => Ok(Self::P),
            "i" | "em" => Ok(Self::I),
            "b" | "strong" => Ok(Self::B),
            "section" => Ok(Self::Section),
            "blockquote" => Ok(Self::Blockquote),
            "div" => Ok(Self::Div),
            "span" => Ok(Self::Span),
            "br" => Ok(Self::Br),
            "hr" => Ok(Self::Hr),
            "a" => Ok(Self::A),
            "tr" => Ok(Self::Tr),
            "image" => Ok(Self::Image),
            _ => Ok(Self::Ignored),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Attribute {
    key: String,
    value: String,
}
impl Attribute {
    fn parse(attr: QAttribute<'_>) -> Result<Self, EpubError> {
        let key = attr.key.into_inner();
        let key = std::str::from_utf8(key)?.to_owned();
        let value = attr.unescape_value()?.into_owned();
        Ok(Self { key, value })
    }
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Default, Debug, Clone)]
pub struct Element {
    variant: ElementVariant,
    attributes: Vec<Attribute>,
    children: Vec<Node>,
}
impl Element {
    fn new(tag: &BytesStart) -> Result<Self, EpubError> {
        let local_name = tag.name().local_name();
        let parsed_name = std::str::from_utf8(local_name.into_inner())?;
        let variant = parsed_name.parse()?;

        let mut attributes = vec![];
        for attr in tag.attributes().filter(|a| a.is_ok()) {
            let attr = attr.map(|a| Attribute::parse(a).unwrap())?;
            attributes.push(attr);
        }

        Ok(Self {
            variant,
            attributes,
            children: vec![],
        })
    }
    pub fn variant(&self) -> ElementVariant {
        self.variant
    }
    pub fn attribute(&self, key: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|a| a.key == key)
    }
    pub fn children(&self) -> &[Node] {
        &self.children
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    Text(String),
    Element(Element),
}

impl Node {
    pub fn new(input: &[u8]) -> Result<Self, EpubError> {
        let t = std::str::from_utf8(input)?;
        let mut reader = quick_xml::Reader::from_str(t);
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"body" => {
                    return extract(e, &mut reader);
                }
                Ok(Event::Eof) => return Err(EpubError::UnexpectedEof),
                Err(e) => return Err(e.into()),
                _ => (),
            }
        }
    }
    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            _ => None,
        }
    }
    pub fn element(&self) -> Option<&Element> {
        if let Self::Element(el) = self {
            return Some(el);
        }
        None
    }
    pub fn iter<'a>(&'a self) -> NodeIterator<'a> {
        NodeIterator {
            stack: vec![(0, self)],
        }
    }
}

pub struct NodeIterator<'a> {
    stack: Vec<(usize, &'a Node)>,
}
impl<'a> Iterator for NodeIterator<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        let (next_child, current_node) = self.stack.pop()?;

        // if it's a branch
        if let Node::Element(el) = current_node {
            // if it has more children then we're done
            if let Some(child_node) = el.children().get(next_child) {
                // push the parent back to the stack
                self.stack.push((next_child + 1, current_node));
                // then push the new child to the stack
                self.stack.push((0, child_node));

                return Some(current_node);
            }
        }

        // if we're here there were no more children on this branch nd we need to find a sibling
        loop {
            // pop parent again to try to find the next child
            let Some((next_child, parent_node)) = self.stack.pop() else {
                break;
            };

            // get the inner element
            let Node::Element(el) = parent_node else {
                panic!("parent node cannot be a leaf");
            };

            // if it has more children then we're done
            if let Some(child_node) = el.children().get(next_child) {
                // push the parent back to the stack
                self.stack.push((next_child + 1, parent_node));
                // then push the new child to the stack
                self.stack.push((0, child_node));
                break;
            }
        }

        Some(current_node)
    }
}

pub fn extract<'a>(tag: &BytesStart, reader: &mut Reader<&[u8]>) -> Result<Node, EpubError> {
    let mut node = Element::new(tag)?;

    for attr in tag.attributes().filter(|a| a.is_ok()) {
        let attr = attr.map(|a| Attribute::parse(a).unwrap())?;
        node.attributes.push(attr);
    }
    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let child = extract(e, reader)?;
                node.children.push(child);
            }
            Ok(Event::Text(text)) => {
                let t = std::str::from_utf8(&text)?;
                let result: String = t.split_whitespace().collect::<Vec<_>>().join(" ");

                if !result.is_empty() {
                    node.children.push(Node::Text(result));
                }
            }
            Ok(Event::Empty(ref e)) => {
                let cur = Element::new(e)?;
                node.children.push(Node::Element(cur));
            }
            Ok(Event::End(_)) => {
                return Ok(Node::Element(node));
            }
            Ok(Event::Eof) => return Err(EpubError::UnexpectedEof),
            Err(e) => {
                return Err(e.into());
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::epub::html::Node;

    #[test]
    fn full_xhtml() {
        let xml = r#"
            <?xml version='1.0' encoding='utf-8'?>
            <html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en">
                <body class="x-ebookmaker x-ebookmaker-2">
                    <div class="blockquot">
                        <p class="nind">“Dear Sir,</p>
                        <p key="value">I must trouble you <i>once</i> more</p>
                        <p>“Yours sincerely,” etc.<br/></p>
                    </div>
                    <p>Addendum<br/></p>
                </body>
            </html>
        "#;
        let node = Node::new(xml.as_bytes()).unwrap();
        let mut node_iter = node.iter().skip(5);

        let elem = node_iter.next().unwrap();
        assert_eq!(
            elem.element().unwrap().attribute("key").unwrap().value,
            "value"
        );

        assert_eq!(
            node_iter.next().unwrap().text().unwrap(),
            "I must trouble you"
        );
    }
}
