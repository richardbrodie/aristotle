use std::{borrow::Cow, str::FromStr};

use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "epub/html.pest"]
struct HtmlParser;

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    HtmlParsing,
    NoNodes,
    UnmatchedRule(Rule),
}
impl From<pest::error::Error<Rule>> for Error {
    fn from(_: pest::error::Error<Rule>) -> Self {
        Self::HtmlParsing
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node<'a> {
    Text(Cow<'a, str>),
    Element(Element<'a>),
    Empty(ElementVariant),
}
impl<'a> Node<'a> {
    pub fn parse(input: &'a [u8]) -> Result<Self, Error> {
        let str = std::str::from_utf8(input).unwrap();
        let html = HtmlParser::parse(Rule::html, str)?;

        for node in html {
            if node.as_rule() == Rule::node_element {
                let res = parse_element_node(node);
                if let Ok(Node::Element(ref el)) = res {
                    if el.variant() != ElementVariant::Ignored {
                        return res;
                    }
                }
            }
        }
        Err(Error::NoNodes)
    }
    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            _ => None,
        }
    }

    pub fn element(&self) -> Option<&Element> {
        match self {
            Self::Element(e) => Some(e),
            _ => None,
        }
    }

    pub fn traverse(&self, v: ElementVariant) -> Option<&Element> {
        match self {
            Self::Element(e) => e.traverse(v),
            _ => None,
        }
    }

    pub fn iter(&'a self) -> NodeIterator<'a> {
        NodeIterator {
            stack: vec![(0, self)],
        }
    }
}

impl<'s> std::fmt::Display for Node<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Node::Element(e) => write!(f, "Element: {:?}", e.variant())?,
            Node::Text(t) => write!(f, "Text: {}", t.trim())?,
            Node::Empty(e) => write!(f, "Empty: {:?}", e)?,
        }
        Ok(())
    }
}

impl<'a> IntoIterator for &'a Node<'a> {
    type Item = &'a Node<'a>;
    type IntoIter = NodeIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        NodeIterator {
            stack: vec![(0, self)],
        }
    }
}

pub struct NodeIterator<'a> {
    stack: Vec<(usize, &'a Node<'a>)>,
}
impl<'a> Iterator for NodeIterator<'a> {
    type Item = &'a Node<'a>;

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

        return Some(current_node);
    }
}

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
    P,
    Section,
    Span,
    #[default]
    Ignored,
}
impl FromStr for ElementVariant {
    type Err = Error;

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
            _ => Ok(Self::Ignored),
        }
    }
}

pub enum ElementType {
    Block,
    Inline,
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Element<'a> {
    variant: ElementVariant,
    attributes: Vec<Attribute<'a>>,
    children: Vec<Node<'a>>,
}
impl<'a> Element<'a> {
    pub fn variant(&self) -> ElementVariant {
        self.variant
    }
    pub fn attribute(&self, key: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|a| a.key == key)
    }
    pub fn children(&self) -> &[Node] {
        &self.children
    }
    pub fn blocktype(&self) -> ElementType {
        match self.variant {
            ElementVariant::A | ElementVariant::B | ElementVariant::I | ElementVariant::Span => {
                ElementType::Inline
            }
            _ => ElementType::Block,
        }
    }
    fn traverse(&self, v: ElementVariant) -> Option<&Element> {
        if self.variant == v {
            return Some(self);
        }
        self.children().iter().filter_map(|c| c.traverse(v)).next()
    }
}

impl<'s> std::fmt::Display for Element<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Element <{:?}> [{}:{}]",
            self.variant(),
            self.attributes.len(),
            self.children.len()
        )?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Attribute<'a> {
    key: Cow<'a, str>,
    value: Cow<'a, str>,
}
impl<'a> Attribute<'a> {
    fn parse(pairs: Pairs<'a, Rule>) -> Result<Self, Error> {
        let mut attribute = Self::default();
        for pair in pairs {
            match pair.as_rule() {
                Rule::attr_key => {
                    attribute.key = Cow::Borrowed(pair.as_str().trim());
                }
                Rule::attr_non_quoted => {
                    attribute.value = Cow::Borrowed(pair.as_str().trim());
                }
                Rule::attr_quoted => {
                    if let Some(inner_pair) = pair.into_inner().next() {
                        match inner_pair.as_rule() {
                            Rule::attr_value => {
                                attribute.value = Cow::Borrowed(inner_pair.as_str().trim());
                            }
                            r => return Err(Error::UnmatchedRule(r)),
                        }
                    }
                }
                r => return Err(Error::UnmatchedRule(r)),
            }
        }
        Ok(attribute)
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn value(&self) -> &str {
        &self.value
    }
}

fn parse_element_node(pair: Pair<'_, Rule>) -> Result<Node, Error> {
    let mut element = Element::default();

    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::el_name | Rule::el_void_name => {
                let v: ElementVariant = pair.as_str().parse()?;
                if let ElementVariant::Br | ElementVariant::Hr = v {
                    return Ok(Node::Empty(v));
                }
                element.variant = v;
            }
            Rule::el_raw_text => println!("Raw text: {}", pair.as_str()),
            Rule::el_process_instruct => {}
            Rule::node_element => {
                if let Ok(mut e) = parse_element_node(pair) {
                    if let Node::Element(ref mut e) = e {
                        if e.variant() == ElementVariant::Ignored {
                            element.children.append(&mut e.children);
                            continue;
                        }
                    }
                    element.children.push(e);
                }
            }
            Rule::node_text => {
                let t = pair.as_str();
                let result: String = t.split_whitespace().collect::<Vec<_>>().join(" ");

                if !result.is_empty() {
                    element.children.push(Node::Text(Cow::Owned(result)));
                }
            }
            Rule::el_normal_end => {}
            Rule::attr => {
                let attr = Attribute::parse(pair.into_inner())?;
                element.attributes.push(attr);
            }

            r => return Err(Error::UnmatchedRule(r)),
        }
    }
    Ok(Node::Element(element))
}

#[cfg(test)]
mod tests {
    use crate::epub::html::{Attribute, Element, ElementVariant, Node};

    #[test]
    fn full_xhtml() {
        let xml = r#"
            <?xml version='1.0' encoding='utf-8'?>
            <html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en">
                <body class="x-ebookmaker x-ebookmaker-2">
                    <div class="blockquot">
                        <p class="nind">“Dear Sir,</p>
                        <p>I must trouble you once more</p>
                        <p>“Yours sincerely,” etc.<br/></p>
                    </div>
                    <p>Addendum<br/></p>
                </body>
            </html>
        "#;
        let node = Node::parse(xml.as_bytes()).unwrap();
        let mut node_iter = node.iter().skip(6);

        assert_eq!(
            node_iter.next().unwrap().text().unwrap(),
            "I must trouble you once more"
        );
    }

    #[test]
    fn full_html() {
        let xml = r#"
        <html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en">
            <body class="x-ebookmaker x-ebookmaker-2">
                <div class="blockquot">
                    <p class="nind">“Dear Sir,</p>
                    <p>I must trouble you once more</p>
                    <p>“Yours sincerely,” etc.<br/></p>
                </div>
                <p>Addendum<br/></p>
            </body>
        </html>
        "#;

        let node = Node::parse(xml.as_bytes()).unwrap();
        let mut node_iter = node.iter().skip(6);

        assert_eq!(
            node_iter.next().unwrap().text().unwrap(),
            "I must trouble you once more"
        );
    }

    #[test]
    fn two_paras() {
        let xml = r#"
                <body class="x-ebookmaker x-ebookmaker-2">
                    <p>
                        R. COLLINS was not left long to the silent contemplation of his
                        successful love;
                    </p>
                    <p>
                    jfdkfdks
                                lremtrkmelmtrkle
                    </p>
                </body>
            "#;

        let node = Node::parse(xml.as_bytes()).unwrap();
        let mut node_iter = node.iter().skip(4);

        assert_eq!(
            node_iter.next().unwrap().text().unwrap(),
            "jfdkfdks lremtrkmelmtrkle"
        );
    }

    #[test]
    fn ignored_tag_nested() {
        let xml = r#"
                <body class="x-ebookmaker x-ebookmaker-2">
                    <div>
                        <xxx>
                            <yyy>
                                <p>
                                    R. COLLINS was not left long to the silent contemplation of his
                                    <i>successful</i>
                                    love;
                                </p>
                            </yyy>
                            <b>ksksks</b>
                        </xxx>
                    </div>
                </body>
            "#;
        let node = Node::parse(xml.as_bytes()).unwrap();
        let mut node_iter = node.iter().skip(2);
        assert_eq!(
            node_iter.next().unwrap().element().unwrap().variant(),
            ElementVariant::P
        );
        assert_eq!(
            node_iter.next().unwrap().text().unwrap(),
            "R. COLLINS was not left long to the silent contemplation of his"
        );
        assert_eq!(
            node_iter.next().unwrap().element().unwrap().variant(),
            ElementVariant::I
        );
        assert_eq!(node_iter.next().unwrap().text().unwrap(), "successful");
    }

    #[test]
    fn heading_and_para() {
        let xml = r#"
                <body class="x-ebookmaker x-ebookmaker-2">
                    <h1>Chapter 1</h1>
                    <p>
                    jfdkfdks
                                lremtrkmelmtrkle
                    </p>
                </body>
            "#;

        let node = Node::parse(xml.as_bytes()).unwrap();
        let mut node_iter = node.iter().skip(1);

        assert_eq!(
            node_iter.next().unwrap().element().unwrap().variant(),
            ElementVariant::H1
        );
        assert_eq!(node_iter.next().unwrap().text().unwrap(), "Chapter 1");
    }

    #[test]
    fn parse_paragraphs_with_div_tag() {
        let xml = r#"
                <body class="x-ebookmaker x-ebookmaker-2">
                 <section class="pg-boilerplate pgheader" id="pg-header" lang="en">
                 <h2 id="pg-header-heading" title="">The Project Gutenberg eBook of <span lang="en" id="pg-title-no-subtitle">Frankenstein; Or, The Modern Prometheus</span></h2>
                 <div>This ebook is for the use of anyone anywhere in the United States and
                 most other parts of the world at no cost and with almost no restrictions
                 whatsoever. You may copy it, give it away or re-use it under the terms
                 of the Project Gutenberg License included with this ebook or online
                 at <a class="reference external" href="https://www.gutenberg.org">www.gutenberg.org</a>. If you are not located in the United States,
                 you will have to check the laws of the country where you are located
                 before using this eBook.</div>
                 </section>
                </body>
            "#;
        let node = Node::parse(xml.as_bytes()).unwrap();
        let mut node_iter = node.iter().skip(7);
        assert_eq!(
            node_iter.next().unwrap().text().unwrap(),
            "This ebook is for the use of anyone anywhere in the United States and most other parts of the world at no cost and with almost no restrictions whatsoever. You may copy it, give it away or re-use it under the terms of the Project Gutenberg License included with this ebook or online at"
        );
    }

    #[test]
    fn parse_paragraphs_with_bold() {
        let xml = r#"
                <body class="x-ebookmaker x-ebookmaker-2">
                    <p>
                        R. COLLINS was not left long to the silent contemplation of his
                        successful love;
                    </p>
                    <p>
                        jfdkfdks
                        <b>lremtrkmelmtrkle</b>
                    </p>
                </body>
            "#;

        let node = Node::parse(xml.as_bytes()).unwrap();
        let mut node_iter = node.iter().skip(5);
        assert_eq!(
            node_iter.next().unwrap().element().unwrap().variant(),
            ElementVariant::B
        );
    }

    #[test]
    fn actual_chapter() {
        let xml = r#"
<body class="x-ebookmaker x-ebookmaker-3"><div class="chapter" id="pgepubid00005">
<h2><a id="letter3"/>Letter 3</h2>
<p class="letter2">
<i>To Mrs. Saville, England.</i>
</p>
<p class="right">
July 7th, 17—.
</p>
<p>
My dear Sister,
</p>
<p>
I write a few lines in haste to say that I am safe—and well advanced on my
voyage. This letter will reach England by a merchantman now on its homeward
voyage from Archangel; more fortunate than I, who may not see my native land,
perhaps, for many years. I am, however, in good spirits: my men are bold and
apparently firm of purpose, nor do the floating sheets of ice that continually
pass us, indicating the dangers of the region towards which we are advancing,
appear to dismay them. We have already reached a very high latitude; but it is
the height of summer, and although not so warm as in England, the southern
gales, which blow us speedily towards those shores which I so ardently desire
to attain, breathe a degree of renovating warmth which I had not expected.
</p>
<p>
No incidents have hitherto befallen us that would make a figure in a letter.
One or two stiff gales and the springing of a leak are accidents which
experienced navigators scarcely remember to record, and I shall be well content
if nothing worse happen to us during our voyage.
</p>
<p>
Adieu, my dear Margaret. Be assured that for my own sake, as well as yours, I
will not rashly encounter danger. I will be cool, persevering, and prudent.
</p>
<p>
But success <i>shall</i> crown my endeavours. Wherefore not? Thus far I have
gone, tracing a secure way over the pathless seas, the very stars themselves
being witnesses and testimonies of my triumph. Why not still proceed over the
untamed yet obedient element? What can stop the determined heart and resolved
will of man?
</p>
<p>
My swelling heart involuntarily pours itself out thus. But I must finish.
Heaven bless my beloved sister!
</p>
<p class="right">
R.W.
</p>
</div>
</body>
"#;

        let node = Node::parse(xml.as_bytes()).unwrap();
        let want = Node::Element(Element {
            variant: ElementVariant::Body,
            attributes: vec![Attribute {
                key: "class".into(),
                value: "x-ebookmaker x-ebookmaker-3".into(),
            }],
            children: vec![Node::Element(Element {
                variant: ElementVariant::Div,
                attributes: vec![
                    Attribute {
                        key: "class".into(),
                        value: "chapter".into(),
                    },
                    Attribute {
                        key: "id".into(),
                        value: "pgepubid00005".into(),
                    },
                ],
                children: vec![
                    Node::Element(Element {
                        variant: ElementVariant::H2,
                        attributes: vec![],
                        children: vec![
                            Node::Element(Element {
                                variant: ElementVariant::A,
                                attributes: vec![Attribute {
                                    key: "id".into(),
                                    value: "letter3".into(),
                                }],
                                children: vec![],
                            }),
                            Node::Text("Letter 3".into()),
                        ],
                    }),
                    Node::Element(Element {
                        variant: ElementVariant::P,
                        attributes: vec![Attribute {
                            key: "class".into(),
                            value: "letter2".into(),
                        }],
                        children: vec![Node::Element(Element {
                            variant: ElementVariant::I,
                            attributes: vec![],
                            children: vec![Node::Text("To Mrs. Saville, England.".into())],
                        })],
                    }),
                    Node::Element(Element {
                        variant: ElementVariant::P,
                        attributes: vec![Attribute {
                            key: "class".into(),
                            value: "right".into(),
                        }],
                        children: vec![Node::Text("July 7th, 17—.".into())],
                    }),
                    Node::Element(Element {
                        variant: ElementVariant::P,
                        attributes: vec![],
                        children: vec![Node::Text("My dear Sister,".into())],
                    }),
                    Node::Element(Element {
                        variant: ElementVariant::P,
                        attributes: vec![],
                        children: vec![Node::Text(
                            "I write a few lines in haste to say that I am safe—and well advanced on my voyage. This letter will reach England by a merchantman now on its homeward voyage from Archangel; more fortunate than I, who may not see my native land, perhaps, for many years. I am, however, in good spirits: my men are bold and apparently firm of purpose, nor do the floating sheets of ice that continually pass us, indicating the dangers of the region towards which we are advancing, appear to dismay them. We have already reached a very high latitude; but it is the height of summer, and although not so warm as in England, the southern gales, which blow us speedily towards those shores which I so ardently desire to attain, breathe a degree of renovating warmth which I had not expected.".into(),
                        )],
                    }),
                    Node::Element(Element {
                        variant: ElementVariant::P,
                        attributes: vec![],
                        children: vec![Node::Text(
                            "No incidents have hitherto befallen us that would make a figure in a letter. One or two stiff gales and the springing of a leak are accidents which experienced navigators scarcely remember to record, and I shall be well content if nothing worse happen to us during our voyage.".into(),
                        )],
                    }),
                    Node::Element(Element {
                        variant: ElementVariant::P,
                        attributes: vec![],
                        children: vec![Node::Text(
                            "Adieu, my dear Margaret. Be assured that for my own sake, as well as yours, I will not rashly encounter danger. I will be cool, persevering, and prudent.".into(),
                        )],
                    }),
                    Node::Element(Element {
                        variant: ElementVariant::P,
                        attributes: vec![],
                        children: vec![
                            Node::Text("But success".into()),
                            Node::Element(Element {
                                variant: ElementVariant::I,
                                attributes: vec![],
                                children: vec![Node::Text("shall".into())],
                            }),
                            Node::Text(
                                "crown my endeavours. Wherefore not? Thus far I have gone, tracing a secure way over the pathless seas, the very stars themselves being witnesses and testimonies of my triumph. Why not still proceed over the untamed yet obedient element? What can stop the determined heart and resolved will of man?".into(),
                            ),
                        ],
                    }),
                    Node::Element(Element {
                        variant: ElementVariant::P,
                        attributes: vec![],
                        children: vec![Node::Text(
                            "My swelling heart involuntarily pours itself out thus. But I must finish. Heaven bless my beloved sister!".into(),
                        )],
                    }),
                    Node::Element(Element {
                        variant: ElementVariant::P,
                        attributes: vec![Attribute {
                            key: "class".into(),
                            value: "right".into(),
                        }],
                        children: vec![Node::Text("R.W.".into())],
                    }),
                ],
            })],
        });
        assert_eq!(node, want);
    }

    #[test]
    fn frontpiece() {
        let xml = r#"
            <body class="x-ebookmaker x-ebookmaker-3">
            <section class="pg-boilerplate pgheader" id="pg-header" lang="en">
            <h2 id="pg-header-heading" title="">The Project Gutenberg eBook of <span lang="en" id="pg-title-no-subtitle">Frankenstein; Or, The Modern Prometheus</span></h2>

            <div>This ebook is for the use of anyone anywhere in the United States and
            most other parts of the world at no cost and with almost no restrictions
            whatsoever. You may copy it, give it away or re-use it under the terms
            of the Project Gutenberg License included with this ebook or online
            at <a class="reference external" href="https://www.gutenberg.org">www.gutenberg.org</a>. If you are not located in the United States,
            you will have to check the laws of the country where you are located
            before using this eBook.</div>

            <div class="container" id="pg-machine-header"><p><strong>Title</strong>: Frankenstein; Or, The Modern Prometheus</p>
            <div id="pg-header-authlist">
            <p><strong>Author</strong>: Mary Wollstonecraft Shelley</p>
            </div>
            <p><strong>Release date</strong>: October 1, 1993 [eBook #84]<br/>
                            Most recently updated: December 2, 2022</p>

            <p><strong>Language</strong>: English</p>

            <p><strong>Credits</strong>: Judith Boss, Christy Phillips, Lynn Hanninen and David Meltzer. HTML version by Al Haines.<br/>
                    Further corrections by Menno de Leeuw.</p>

            </div><div id="pg-start-separator">
            <span>*** START OF THE PROJECT GUTENBERG EBOOK FRANKENSTEIN; OR, THE MODERN PROMETHEUS ***</span>
            </div></section><div style="margin-top:2em; margin-bottom:4em"/>
            <h1 id="pgepubid00000">Frankenstein;</h1>
            <h3 id="pgepubid00001">or, the Modern Prometheus</h3>
            <h2 class="no-break">by Mary Wollstonecraft (Godwin) Shelley</h2>
            <hr/>
            </body>
        "#;

        let want = Node::Element(Element {
            variant: ElementVariant::Body,
            attributes: vec![Attribute {
                key: "class".into(),
                value: "x-ebookmaker x-ebookmaker-3".into(),
            }],
            children: vec![
                Node::Element(Element {
                    variant: ElementVariant::Section,
                    attributes: vec![
                        Attribute {
                            key: "class".into(),
                            value: "pg-boilerplate pgheader".into(),
                        },
                        Attribute {
                            key: "id".into(),
                            value: "pg-header".into(),
                        },
                        Attribute {
                            key: "lang".into(),
                            value: "en".into(),
                        },
                    ],
                    children: vec![
                        Node::Element(Element {
                            variant: ElementVariant::H2,
                            attributes: vec![
                                Attribute {
                                    key: "id".into(),
                                    value: "pg-header-heading".into(),
                                },
                                Attribute {
                                    key: "title".into(),
                                    value: "".into(),
                                },
                            ],
                            children: vec![
                                Node::Text("The Project Gutenberg eBook of".into()),
                                Node::Element(Element {
                                    variant: ElementVariant::Span,
                                    attributes: vec![
                                        Attribute {
                                            key: "lang".into(),
                                            value: "en".into(),
                                        },
                                        Attribute {
                                            key: "id".into(),
                                            value: "pg-title-no-subtitle".into(),
                                        },
                                    ],
                                    children: vec![Node::Text("Frankenstein; Or, The Modern Prometheus".into())],
                                }),
                            ],
                        }),
                        Node::Element(Element {
                            variant: ElementVariant::Div,
                            attributes: vec![],
                            children: vec![
                                Node::Text(
                                    "This ebook is for the use of anyone anywhere in the United States and most other parts of the world at no cost and with almost no restrictions whatsoever. You may copy it, give it away or re-use it under the terms of the Project Gutenberg License included with this ebook or online at".into(),
                                ),
                                Node::Element(Element {
                                    variant: ElementVariant::A,
                                    attributes: vec![
                                        Attribute {
                                            key: "class".into(),
                                            value: "reference external".into(),
                                        },
                                        Attribute {
                                            key: "href".into(),
                                            value: "https://www.gutenberg.org".into(),
                                        },
                                    ],
                                    children: vec![Node::Text("www.gutenberg.org".into())],
                                }),
                                Node::Text(
                                    ". If you are not located in the United States, you will have to check the laws of the country where you are located before using this eBook.".into(),
                                ),
                            ],
                        }),
                        Node::Element(Element {
                            variant: ElementVariant::Div,
                            attributes: vec![
                                Attribute {
                                    key: "class".into(),
                                    value: "container".into(),
                                },
                                Attribute {
                                    key: "id".into(),
                                    value: "pg-machine-header".into(),
                                },
                            ],
                            children: vec![
                                Node::Element(Element {
                                    variant: ElementVariant::P,
                                    attributes: vec![],
                                    children: vec![
                                        Node::Element(Element {
                                            variant: ElementVariant::B,
                                            attributes: vec![],
                                            children: vec![Node::Text("Title".into())],
                                        }),
                                        Node::Text(": Frankenstein; Or, The Modern Prometheus".into()),
                                    ],
                                }),
                                Node::Element(Element {
                                    variant: ElementVariant::Div,
                                    attributes: vec![Attribute {
                                        key: "id".into(),
                                        value: "pg-header-authlist".into(),
                                    }],
                                    children: vec![Node::Element(Element {
                                        variant: ElementVariant::P,
                                        attributes: vec![],
                                        children: vec![
                                            Node::Element(Element {
                                                variant: ElementVariant::B,
                                                attributes: vec![],
                                                children: vec![Node::Text("Author".into())],
                                            }),
                                            Node::Text(": Mary Wollstonecraft Shelley".into()),
                                        ],
                                    })],
                                }),
                                Node::Element(Element {
                                    variant: ElementVariant::P,
                                    attributes: vec![],
                                    children: vec![
                                        Node::Element(Element {
                                            variant: ElementVariant::B,
                                            attributes: vec![],
                                            children: vec![Node::Text("Release date".into())],
                                        }),
                                        Node::Text(": October 1, 1993 [eBook #84]".into()),
                                        Node::Empty(ElementVariant::Br),
                                        Node::Text("Most recently updated: December 2, 2022".into()),
                                    ],
                                }),
                                Node::Element(Element {
                                    variant: ElementVariant::P,
                                    attributes: vec![],
                                    children: vec![
                                        Node::Element(Element {
                                            variant: ElementVariant::B,
                                            attributes: vec![],
                                            children: vec![Node::Text("Language".into())],
                                        }),
                                        Node::Text(": English".into()),
                                    ],
                                }),
                                Node::Element(Element {
                                    variant: ElementVariant::P,
                                    attributes: vec![],
                                    children: vec![
                                        Node::Element(Element {
                                            variant: ElementVariant::B,
                                            attributes: vec![],
                                            children: vec![Node::Text("Credits".into())],
                                        }),
                                        Node::Text(
                                            ": Judith Boss, Christy Phillips, Lynn Hanninen and David Meltzer. HTML version by Al Haines.".into(),
                                        ),
                                        Node::Empty(ElementVariant::Br),
                                        Node::Text("Further corrections by Menno de Leeuw.".into()),
                                    ],
                                }),
                            ],
                        }),
                        Node::Element(Element {
                            variant: ElementVariant::Div,
                            attributes: vec![Attribute {
                                key: "id".into(),
                                value: "pg-start-separator".into(),
                            }],
                            children: vec![Node::Element(Element {
                                variant: ElementVariant::Span,
                                attributes: vec![],
                                children: vec![Node::Text(
                                    "*** START OF THE PROJECT GUTENBERG EBOOK FRANKENSTEIN; OR, THE MODERN PROMETHEUS ***".into(),
                                )],
                            })],
                        }),
                    ],
                }),
                Node::Element(Element {
                    variant: ElementVariant::Div,
                    attributes: vec![Attribute {
                        key: "style".into(),
                        value: "margin-top:2em; margin-bottom:4em".into(),
                    }],
                    children: vec![],
                }),
                Node::Element(Element {
                    variant: ElementVariant::H1,
                    attributes: vec![Attribute {
                        key: "id".into(),
                        value: "pgepubid00000".into(),
                    }],
                    children: vec![Node::Text("Frankenstein;".into())],
                }),
                Node::Element(Element {
                    variant: ElementVariant::H3,
                    attributes: vec![Attribute {
                        key: "id".into(),
                        value: "pgepubid00001".into(),
                    }],
                    children: vec![Node::Text("or, the Modern Prometheus".into())],
                }),
                Node::Element(Element {
                    variant: ElementVariant::H2,
                    attributes: vec![Attribute {
                        key: "class".into(),
                        value: "no-break".into(),
                    }],
                    children: vec![Node::Text("by Mary Wollstonecraft (Godwin) Shelley".into())],
                }),
                Node::Empty(ElementVariant::Hr),
            ],
        });
        let got = Node::parse(xml.as_bytes()).unwrap();
        assert_eq!(got, want);
    }
}
