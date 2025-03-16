use quick_xml::{
    events::{BytesText, Event},
    Reader,
};

use super::error::ContentError;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct TextElement {
    pub text: String,
    pub style: TextStyle,
}
impl TextElement {
    fn words(&self) -> usize {
        self.text.split_whitespace().count()
    }
}

#[derive(Debug, PartialEq, Clone)]
enum ParagraphElement {
    Text(TextElement),
    LineBreak,
}
impl ParagraphElement {
    fn text(text: String, style: TextStyle) -> Self {
        Self::Text(TextElement { text, style })
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum TextStyle {
    #[default]
    Regular,
    Italic,
    Bold,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TopLevelElement {
    Heading(Vec<String>),
    Paragraph(Vec<ParagraphElement>),
    Linebreak,
}
// impl TopLevelElement {
//     fn words(&self) -> usize {
//         match self {
//             TopLevelElement::Paragraph(p) => p.iter().fold(0, |acc, e| acc + e.words()),
//             TopLevelElement::Heading(s) => s
//                 .iter()
//                 .fold(0, |acc, e| acc + e.split_whitespace().count()),
//             TopLevelElement::Linebreak => 0,
//         }
//     }
// }

#[derive(Debug, PartialEq, Default)]
pub struct Content {
    id: String,
    pub tokens: Vec<TopLevelElement>,
}
impl Content {
    pub fn new(id: &str, text: &[u8]) -> Result<Self, ContentError> {
        let tokens = tokenize(text)?;
        Ok(Self {
            id: id.to_owned(),
            tokens,
        })
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn tokens(&self) -> &[TopLevelElement] {
        &self.tokens
    }
    // pub fn words(&self) -> usize {
    //     self.tokens().iter().fold(0, |acc, e| acc + e.words())
    // }
}

#[derive(Debug, PartialEq, Clone)]
enum Tag {
    Paragraph,
    Header,
    Break,
    Body,
    Span,
    Bold,
    Italic,
    Div,
    Ignored(String),
}

// fn tokenize(data: &[u8]) -> Result<Vec<TopLevelElement>, ContentError> {
//     let str_content = std::str::from_utf8(data).unwrap();
//     let mut reader = Reader::from_str(str_content);
//     let mut buf = vec![];

//     // fast forward to a specific node
//     loop {
//         match reader.read_event_into(&mut buf) {
//             Ok(Event::Start(ref e)) => {
//                 if e.name().as_ref() == b"body" {
//                     break;
//                 }
//             }
//             Err(e) => return Err(ContentError::Other(e.to_string())),
//             Ok(Event::Eof) => return Err(ContentError::UnexpectedEof),
//             _ => (),
//         }
//     }

//     let mut tokens = vec![];
//     // let mut tag = Tag::Body;
//     let mut tags = vec![];
//     let mut style = TextStyle::Regular;
//     let mut top = None;

//     loop {
//         match reader.read_event_into(&mut buf) {
//             Ok(Event::Start(ref e)) => {
//                 let start_tag = match_tag(e.name().as_ref());
//                 match &start_tag {
//                     Tag::Header => {
//                         if top.is_none() {
//                             top = Some(TopLevelElement::Heading(Vec::new()));
//                         }
//                     }
//                     Tag::Paragraph => {
//                         if top.is_none() {
//                             top = Some(TopLevelElement::Paragraph(Vec::new()));
//                         }
//                     }
//                     Tag::Bold | Tag::Strong => style = TextStyle::Bold,
//                     Tag::Italic => style = TextStyle::Italic,
//                     // t => println!("opening some other tag: {:?}", t),
//                     _ => (),
//                 }
//                 tags.push(start_tag);
//             }
//             Ok(Event::End(ref e)) => {
//                 let end_tag = match_tag(e.name().as_ref());
//                 match end_tag {
//                     Tag::Body => break,
//                     Tag::Header => {
//                         let t = top.expect("closing an unopened header");
//                         tokens.push(t);
//                         top = None;
//                     }
//                     Tag::Paragraph => {
//                         let t = top.expect("closing an unopened para");
//                         tokens.push(t);
//                         top = None;
//                     }
//                     // Tag::Div => {
//                     //     if let Some(TopLevelElement::Paragraph(p)) = &top {
//                     //         println!("closing a para created by a div");
//                     //         if !p.is_empty() {
//                     //             let t = top.unwrap();
//                     //             tokens.push(t);
//                     //         }
//                     //         top = None;
//                     //     }
//                     // }
//                     Tag::Bold | Tag::Italic | Tag::Strong => style = TextStyle::Regular,
//                     // t => println!("closing some other tag: {:?}", t),
//                     _ => (),
//                 }
//             }
//             Ok(Event::Empty(ref e)) => {
//                 let empty_tag = match_tag(e.name().as_ref());
//                 match empty_tag {
//                     Tag::Break => match &mut top {
//                         Some(TopLevelElement::Paragraph(p)) => p.push(ParagraphElement::LineBreak),
//                         None => tokens.push(TopLevelElement::Linebreak),
//                         _ => (),
//                     },
//                     // t => println!("unhandled empty tag: {:?}", t),
//                     _ => (),
//                 }
//             }
//             Ok(Event::Text(t)) => {
//                 let text = extract_text(t);
//                 if text.is_empty() {
//                     continue;
//                 }

//                 match &mut top {
//                     Some(TopLevelElement::Heading(h)) => h.push(text),
//                     Some(TopLevelElement::Paragraph(p)) => {
//                         p.push(ParagraphElement::Text(TextElement { text, style }))
//                     }
//                     None => panic!("text with no top level element: {}", text),
//                     t => panic!("unhandled text tag: {:?}", t),
//                 }
//             }
//             Err(e) => panic!("error: {:?}", e),
//             Ok(Event::Eof) => break,
//             t => println!("unmatched: {:?}", t),
//         }
//     }
//     Ok(tokens)
// }
//
fn tokenize(data: &[u8]) -> Result<Vec<TopLevelElement>, ContentError> {
    let str_content = std::str::from_utf8(data).unwrap();
    let mut reader = Reader::from_str(str_content);
    let mut buf = vec![];

    // fast forward to a specific node
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"body" {
                    break;
                }
            }
            Err(e) => return Err(ContentError::Other(e.to_string())),
            Ok(Event::Eof) => return Err(ContentError::UnexpectedEof),
            _ => (),
        }
    }

    let mut tokens = vec![];
    let mut tags = vec![];
    let mut top: Option<TopLevelElement> = None;
    let mut open_tags = vec![];

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let start_tag = match_tag(e.name().as_ref());
                tags.push(start_tag);
            }
            Ok(Event::End(ref e)) => {
                let end_tag = match_tag(e.name().as_ref());

                match end_tag {
                    Tag::Body => break,
                    Tag::Italic | Tag::Bold => {}
                    _ => {
                        println!("closing end tag: {:?} {:?} {:?}", end_tag, open_tags, tags);
                        if tags == open_tags {
                            let Some(elem) = top.take() else {
                                panic!("no top element");
                            };
                            println!("finished top element: {:?}", &elem);
                            tokens.push(elem);
                        }
                    }
                }
                let _ = tags.pop();
            }
            Ok(Event::Empty(ref e)) => {
                let empty_tag = match_tag(e.name().as_ref());
                match empty_tag {
                    Tag::Break => match &mut top {
                        Some(TopLevelElement::Paragraph(p)) => p.push(ParagraphElement::LineBreak),
                        None => tokens.push(TopLevelElement::Linebreak),
                        _ => (),
                    },
                    t => println!("unhandled empty tag: {:?}", t),
                    _ => (),
                }
            }
            Ok(Event::Text(t)) => {
                let text = extract_text(t);
                if text.is_empty() {
                    continue;
                }

                match &mut top {
                    Some(TopLevelElement::Heading(h)) => {
                        h.push(text);
                        continue;
                    }
                    Some(TopLevelElement::Paragraph(p)) => {
                        let style = tags
                            .iter()
                            .find_map(|t| match t {
                                Tag::Italic => Some(TextStyle::Italic),
                                Tag::Bold => Some(TextStyle::Bold),
                                _ => None,
                            })
                            .unwrap_or(TextStyle::Regular);
                        let elem = ParagraphElement::text(text, style);
                        p.push(elem);
                        continue;
                    }
                    None => {
                        println!("text with no top element");
                        let made_top = make_top(&tags, text);
                        open_tags = tags.clone();
                        top = Some(made_top);
                    }
                    _ => (),
                }
            }
            Err(e) => panic!("error: {:?}", e),
            Ok(Event::Eof) => break,
            t => println!("unmatched: {:?}", t),
        }
    }
    Ok(tokens)
}

fn make_top(tags: &[Tag], text: String) -> TopLevelElement {
    let mut style = TextStyle::Regular;
    for i in tags.iter().rev() {
        match i {
            Tag::Italic => style = TextStyle::Italic,
            Tag::Bold => style = TextStyle::Bold,
            Tag::Header => {
                println!("creating header");
                return TopLevelElement::Heading(vec![text]);
            }
            Tag::Paragraph => {
                println!("creating para");
                let elem = ParagraphElement::text(text, style);
                return TopLevelElement::Paragraph(vec![elem]);
            }
            t => println!("unclear tag: {:?}", t),
        }
    }
    println!("no top yet, para?");
    let elem = ParagraphElement::text(text, style);
    return TopLevelElement::Paragraph(vec![elem]);
}

fn match_tag(raw: &[u8]) -> Tag {
    match raw {
        b"p" => Tag::Paragraph,
        b"b" | b"strong" => Tag::Bold,
        b"i" | b"em" => Tag::Italic,
        b"br" => Tag::Break,
        b"body" => Tag::Body,
        b"span" => Tag::Span,
        b"div" => Tag::Div,
        h if h[0] == 'h' as u8 && h[1] != 'r' as u8 => Tag::Header,
        t => {
            let s = std::str::from_utf8(t).unwrap();
            Tag::Ignored(s.to_owned())
        }
    }
}

fn extract_text(t: BytesText) -> String {
    let t = t.unescape().unwrap();
    let mut result = String::with_capacity(t.len());
    t.split_whitespace().for_each(|w| {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(w);
    });
    result
}

#[cfg(test)]
mod tests {
    use super::{tokenize, ParagraphElement, TextElement, TextStyle, TopLevelElement};

    #[test]
    fn parse_paragraphs() {
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
        let want = vec![
                TopLevelElement::Paragraph(vec![
                    ParagraphElement::Text(TextElement{
                        text: "R. COLLINS was not left long to the silent contemplation of his successful love;".to_owned(),
                        style: TextStyle::Regular,
                    })
                ]),
                TopLevelElement::Paragraph(vec![
                    ParagraphElement::Text(TextElement{
                        text:"jfdkfdks lremtrkmelmtrkle".to_owned(),
                        style: TextStyle::Regular,
                    })
                ])
            ];
        let got = tokenize(xml.as_bytes()).unwrap();
        assert_eq!(got, want);
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
        let want = vec![
            TopLevelElement::Paragraph(vec![
                ParagraphElement::Text(TextElement{
                    text: "R. COLLINS was not left long to the silent contemplation of his successful love;".to_owned(),
                    style: TextStyle::Regular,
                })
            ]),
            TopLevelElement::Paragraph(vec![
                ParagraphElement::Text(TextElement{
                    text:"jfdkfdks".to_owned(),
                    style: TextStyle::Regular,
                }),
                ParagraphElement::Text(TextElement{
                    text:"lremtrkmelmtrkle".to_owned(),
                    style: TextStyle::Bold,
                })
            ])
        ];
        let got = tokenize(xml.as_bytes()).unwrap();
        assert_eq!(got, want);
    }

    #[test]
    fn parse_paragraphs_with_italic() {
        let xml = r#"
        <body class="x-ebookmaker x-ebookmaker-2"><div class="chapter" id="pgepubid00005">
        <h2><a id="letter3"/>Letter 3</h2>
        <p class="letter2">
        <i>To Mrs. Saville, England.</i>
        </p></div>
        </body>
                "#;
        let want = vec![
            TopLevelElement::Heading(vec!["Letter 3".to_owned()]),
            TopLevelElement::Paragraph(vec![ParagraphElement::Text(TextElement {
                text: "To Mrs. Saville, England.".to_owned(),
                style: TextStyle::Italic,
            })]),
        ];
        let got = tokenize(xml.as_bytes()).unwrap();
        assert_eq!(got, want);
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
        let want = vec![
            TopLevelElement::Heading(vec!["The Project Gutenberg eBook of".to_owned(), "Frankenstein; Or, The Modern Prometheus".to_owned()]),
            TopLevelElement::Paragraph(vec![
                ParagraphElement::Text(TextElement { text: "This ebook is for the use of anyone anywhere in the United States and most other parts of the world at no cost and with almost no restrictions whatsoever. You may copy it, give it away or re-use it under the terms of the Project Gutenberg License included with this ebook or online at".to_owned(), style: TextStyle::Regular }),
                ParagraphElement::Text(TextElement { text: "www.gutenberg.org".to_owned(), style: TextStyle::Regular }),
                ParagraphElement::Text(TextElement { text: ". If you are not located in the United States, you will have to check the laws of the country where you are located before using this eBook.".to_owned(), style: TextStyle::Regular })
            ]),
        ];
        let got = tokenize(xml.as_bytes()).unwrap();
        assert_eq!(got, want);
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

        let want = vec![
            TopLevelElement::Heading(vec!["Letter 3".to_owned()]),
            TopLevelElement::Paragraph(vec![ParagraphElement::Text(TextElement {
                text: "To Mrs. Saville, England.".to_owned(),
                style: TextStyle::Italic,
            })]),
            TopLevelElement::Paragraph(vec![ParagraphElement::Text(TextElement {
                text: "July 7th, 17—.".to_owned(),
                style: TextStyle::Regular,
            })]),
            TopLevelElement::Paragraph(vec![ParagraphElement::Text(TextElement {
                text: "My dear Sister,".to_owned(),
                style: TextStyle::Regular,
            })]),
            TopLevelElement::Paragraph(vec![ParagraphElement::Text(TextElement {
                text: "I write a few lines in haste to say that I am safe—and well advanced on my voyage. This letter will reach England by a merchantman now on its homeward voyage from Archangel; more fortunate than I, who may not see my native land, perhaps, for many years. I am, however, in good spirits: my men are bold and apparently firm of purpose, nor do the floating sheets of ice that continually pass us, indicating the dangers of the region towards which we are advancing, appear to dismay them. We have already reached a very high latitude; but it is the height of summer, and although not so warm as in England, the southern gales, which blow us speedily towards those shores which I so ardently desire to attain, breathe a degree of renovating warmth which I had not expected."
                    .to_owned(),
                style: TextStyle::Regular,
            })]),
            TopLevelElement::Paragraph(vec![ParagraphElement::Text(TextElement {
                text: "No incidents have hitherto befallen us that would make a figure in a letter. One or two stiff gales and the springing of a leak are accidents which experienced navigators scarcely remember to record, and I shall be well content if nothing worse happen to us during our voyage."
                    .to_owned(),
                style: TextStyle::Regular,
            })]),
            TopLevelElement::Paragraph(vec![ParagraphElement::Text(TextElement {
                text:
                    "Adieu, my dear Margaret. Be assured that for my own sake, as well as yours, I will not rashly encounter danger. I will be cool, persevering, and prudent."
                        .to_owned(),
                style: TextStyle::Regular,
            })]),
            TopLevelElement::Paragraph(vec![
                ParagraphElement::Text(TextElement {
                    text: "But success".to_owned(),
                    style: TextStyle::Regular,
                }),
                ParagraphElement::Text(TextElement {
                    text: "shall".to_owned(),
                    style: TextStyle::Italic,
                }),
                ParagraphElement::Text(TextElement {
                    text: "crown my endeavours. Wherefore not? Thus far I have gone, tracing a secure way over the pathless seas, the very stars themselves being witnesses and testimonies of my triumph. Why not still proceed over the untamed yet obedient element? What can stop the determined heart and resolved will of man?"
                        .to_owned(),
                    style: TextStyle::Regular,
                }),
            ]),
            TopLevelElement::Paragraph(vec![ParagraphElement::Text(TextElement {
                text: "My swelling heart involuntarily pours itself out thus. But I must finish. Heaven bless my beloved sister!"
                    .to_owned(),
                style: TextStyle::Regular,
            })]),
            TopLevelElement::Paragraph(vec![ParagraphElement::Text(TextElement {
                text: "R.W.".to_owned(),
                style: TextStyle::Regular,
            })]),
        ];
        let got = tokenize(xml.as_bytes()).unwrap();

        for i in 0..got.len() {
            assert_eq!(got[i], want[i]);
        }
    }

    #[test]
    fn title() {
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
            </div></section>
            </body>
            "#;

        let want = vec![
            TopLevelElement::Heading(vec!["The Project Gutenberg eBook of".to_owned(), "Frankenstein; Or, The Modern Prometheus".to_owned()]),
            TopLevelElement::Paragraph(vec![
                ParagraphElement::Text(TextElement { text: "This ebook is for the use of anyone anywhere in the United States and most other parts of the world at no cost and with almost no restrictions whatsoever. You may copy it, give it away or re-use it under the terms of the Project Gutenberg License included with this ebook or online at".to_owned(), style: TextStyle::Regular }),
                ParagraphElement::Text(TextElement { text: "www.gutenberg.org".to_owned(), style: TextStyle::Regular }),
                ParagraphElement::Text(TextElement { text: ". If you are not located in the United States, you will have to check the laws of the country where you are located before using this eBook.".to_owned(), style: TextStyle::Regular })
            ]),
            TopLevelElement::Paragraph(vec![
                ParagraphElement::Text(TextElement { text: "Title".to_owned(), style: TextStyle::Bold }),
                ParagraphElement::Text(TextElement { text: ": Frankenstein; Or, The Modern Prometheus".to_owned(), style: TextStyle::Regular }),
            ]),
        ];
        let got = tokenize(xml.as_bytes()).unwrap();

        for i in 0..got.len() {
            assert_eq!(got[i], want[i]);
        }
    }

    // #[test]
    // fn frontpiece() {
    //     let xml = r#"
    //         <body class="x-ebookmaker x-ebookmaker-3">
    //         <section class="pg-boilerplate pgheader" id="pg-header" lang="en">
    //         <h2 id="pg-header-heading" title="">The Project Gutenberg eBook of <span lang="en" id="pg-title-no-subtitle">Frankenstein; Or, The Modern Prometheus</span></h2>

    //         <div>This ebook is for the use of anyone anywhere in the United States and
    //         most other parts of the world at no cost and with almost no restrictions
    //         whatsoever. You may copy it, give it away or re-use it under the terms
    //         of the Project Gutenberg License included with this ebook or online
    //         at <a class="reference external" href="https://www.gutenberg.org">www.gutenberg.org</a>. If you are not located in the United States,
    //         you will have to check the laws of the country where you are located
    //         before using this eBook.</div>

    //         <div class="container" id="pg-machine-header"><p><strong>Title</strong>: Frankenstein; Or, The Modern Prometheus</p>
    //         <div id="pg-header-authlist">
    //         <p><strong>Author</strong>: Mary Wollstonecraft Shelley</p>
    //         </div>
    //         <p><strong>Release date</strong>: October 1, 1993 [eBook #84]<br/>
    //                         Most recently updated: December 2, 2022</p>

    //         <p><strong>Language</strong>: English</p>

    //         <p><strong>Credits</strong>: Judith Boss, Christy Phillips, Lynn Hanninen and David Meltzer. HTML version by Al Haines.<br/>
    //                 Further corrections by Menno de Leeuw.</p>

    //         </div><div id="pg-start-separator">
    //         <span>*** START OF THE PROJECT GUTENBERG EBOOK FRANKENSTEIN; OR, THE MODERN PROMETHEUS ***</span>
    //         </div></section><div style="margin-top:2em; margin-bottom:4em"/>
    //         <h1 id="pgepubid00000">Frankenstein;</h1>
    //         <h3 id="pgepubid00001">or, the Modern Prometheus</h3>
    //         <h2 class="no-break">by Mary Wollstonecraft (Godwin) Shelley</h2>
    //         <hr/>
    //         </body>
    //         "#;

    //     let want = vec![
    //         TopLevelElement::Heading(vec!["The Project Gutenberg eBook of".to_owned(), "Frankenstein; Or, The Modern Prometheus".to_owned()]),
    //         TopLevelElement::Paragraph(vec![
    //             ParagraphElement::Text(TextElement { text: "This ebook is for the use of anyone anywhere in the United States and most other parts of the world at no cost and with almost no restrictions whatsoever. You may copy it, give it away or re-use it under the terms of the Project Gutenberg License included with this ebook or online at".to_owned(), style: TextStyle::Regular }),
    //             ParagraphElement::Text(TextElement { text: "www.gutenberg.org".to_owned(), style: TextStyle::Regular }),
    //             ParagraphElement::Text(TextElement { text: ". If you are not located in the United States, you will have to check the laws of the country where you are located before using this eBook.".to_owned(), style: TextStyle::Regular })
    //         ]),
    //         TopLevelElement::Paragraph(vec![
    //             ParagraphElement::Text(TextElement { text: "Title".to_owned(), style: TextStyle::Bold }),
    //             ParagraphElement::Text(TextElement { text: ": Frankenstein; Or, The Modern Prometheus".to_owned(), style: TextStyle::Regular }),
    //         ]),
    //         TopLevelElement::Paragraph(vec![
    //             ParagraphElement::Text(TextElement { text: "Author".to_owned(), style: TextStyle::Bold }),
    //             ParagraphElement::Text(TextElement { text: ": Mary Wollstonecraft Shelley".to_owned(), style: TextStyle::Regular }),
    //         ]),
    //         TopLevelElement::Paragraph(vec![
    //             ParagraphElement::Text(TextElement { text: "Release date".to_owned(), style: TextStyle::Bold }),
    //             ParagraphElement::Text(TextElement { text: ": October 1, 1993 [eBook #84]".to_owned(), style: TextStyle::Regular }),
    //             ParagraphElement::LineBreak,
    //             ParagraphElement::Text(TextElement { text: "Most recently updated: December 2, 2022".to_owned(), style: TextStyle::Regular }),
    //         ]),
    //         TopLevelElement::Paragraph(vec![
    //             ParagraphElement::Text(TextElement { text: "Language".to_owned(), style: TextStyle::Bold }),
    //             ParagraphElement::Text(TextElement { text: ": English".to_owned(), style: TextStyle::Regular }),
    //         ]),
    //         TopLevelElement::Paragraph(vec![
    //             ParagraphElement::Text(TextElement { text: "Credits".to_owned(), style: TextStyle::Bold }),
    //             ParagraphElement::Text(TextElement { text: ": Judith Boss, Christy Phillips, Lynn Hanninen and David Meltzer. HTML version by Al Haines.".to_owned(), style: TextStyle::Regular }),
    //             ParagraphElement::LineBreak,
    //             ParagraphElement::Text(TextElement { text: "Further corrections by Menno de Leeuw.".to_owned(), style: TextStyle::Regular }),
    //         ]),
    //         TopLevelElement::Paragraph(vec![
    //             ParagraphElement::Text(TextElement { text: "*** START OF THE PROJECT GUTENBERG EBOOK FRANKENSTEIN; OR, THE MODERN PROMETHEUS ***".to_owned(), style: TextStyle::Regular }),
    //         ]),
    //         TopLevelElement::Heading(vec![
    //             "Frankenstein;".to_owned(),
    //         ]),
    //         TopLevelElement::Heading(vec![
    //             "or, the Modern Prometheus".to_owned(),
    //         ]),
    //         TopLevelElement::Heading(vec![
    //             "by Mary Wollstonecraft (Godwin) Shelley".to_owned(),
    //         ])
    //     ];
    //     let got = tokenize(xml.as_bytes()).unwrap();

    //     for i in 0..got.len() {
    //         assert_eq!(got[i], want[i]);
    //     }
    // }

    // #[test]
    // fn parse_paragraphs_words() {
    //     let xml = r#"
    // <?xml version='1.0' encoding='utf-8'?>
    // <!DOCTYPE html PUBLIC '-//W3C//DTD XHTML 1.1//EN' 'http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd'>
    // <body class="x-ebookmaker x-ebookmaker-2">
    //     <p>
    //         R. COLLINS was not left long to the silent contemplation of his
    //         successful love;
    //     </p>
    //     <p>
    //     jfdkfdks
    //                 lremtrkmelmtrkle
    //     </p>
    // </body>
    //         "#;
    //     let want = 16;
    //     let got = Content::new("", xml.as_bytes()).unwrap();
    //     assert_eq!(got.words(), want);
    // }

    // #[test]
    // fn parse_paragraphs_with_bold_words() {
    //     let xml = r#"
    //     <?xml version='1.0' encoding='utf-8'?>
    //     <!DOCTYPE html PUBLIC '-//W3C//DTD XHTML 1.1//EN' 'http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd'>
    //     <body class="x-ebookmaker x-ebookmaker-2">
    //         <p>
    //             R. COLLINS was not left long to the silent contemplation of his
    //             successful love;
    //         </p>
    //         <p>
    //         jfdkfdks
    //                     <b>lremtrkmelmtrkle</b>
    //         </p>
    //     </body>
    //             "#;
    //     let want = 16;
    //     let got = Content::new("", xml.as_bytes()).unwrap();
    //     assert_eq!(got.words(), want);
    // }
}
