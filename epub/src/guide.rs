use crate::Error;

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Reference {
    ref_type: String,
    title: String,
    href: String,
}
impl Reference {
    pub fn parse(node: roxmltree::Node<'_, '_>) -> Result<Reference, Error> {
        let mut href = None;
        let mut ref_type = None;
        let mut title = None;
        for attr in node.attributes() {
            match attr.name() {
                "href" => href = Some(attr.value().to_owned()),
                "title" => title = Some(attr.value().to_owned()),
                "type" => ref_type = Some(attr.value().to_owned()),
                _ => {}
            }
        }
        Ok(Reference {
            ref_type: ref_type.ok_or(Error::XmlField)?,
            title: title.ok_or(Error::XmlField)?,
            href: href.ok_or(Error::XmlField)?,
        })
    }
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Guide {
    references: Vec<Reference>,
}
impl Guide {
    pub fn extract(doc: &roxmltree::Document<'_>) -> Result<Option<Guide>, Error> {
        let guide_node = doc.descendants().find(|n| n.has_tag_name("guide"));
        if guide_node.is_none() {
            return Ok(None);
        }

        let references: Vec<_> = guide_node
            .unwrap()
            .children()
            .filter(|node| node.has_tag_name("reference"))
            .flat_map(Reference::parse)
            .collect();
        Ok(Some(Guide { references }))
    }
}

#[cfg(test)]
mod tests {
    use super::Reference;

    #[test]
    fn reference_correct() {
        let xml =
            "<reference type='loi' title='List Of Illustrations' href='appendix.xhtml#figures' />";
        let doc = roxmltree::Document::parse(xml).unwrap();
        let n = doc.root_element();
        let result = Reference::parse(n);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.title, "List Of Illustrations");
    }

    #[test]
    fn reference_missing_field() {
        let xml = "<reference title='List Of Illustrations' href='appendix.xhtml#figures' />";
        let doc = roxmltree::Document::parse(xml).unwrap();
        let n = doc.root_element();
        let result = Reference::parse(n);
        assert!(result.is_err());
    }
}
