use std::path::Path;

use super::{manifest::Manifest, spine::Spine};

#[derive(Debug, Default)]
pub struct Index {
    elements: Vec<IndexElement>,
}
impl Index {
    pub fn new<P: AsRef<Path>>(manifest: Manifest, spine: Spine, contents_dir: P) -> Self {
        let mut elements = vec![];
        for s in spine.items.into_iter() {
            if let Some(m) = manifest.item(&s) {
                let path = contents_dir.as_ref().join(&m.href);
                let path = path.to_str().unwrap();
                let e = IndexElement {
                    id: s,
                    path: path.to_owned(),
                };
                elements.push(e);
            }
        }
        Self { elements }
    }

    pub fn iter(&self) -> impl Iterator<Item = &IndexElement> {
        self.elements.iter()
    }
    pub fn element(&self, id: &str) -> Option<&IndexElement> {
        self.elements.iter().find(|i| i.id() == id)
    }
    pub fn first(&self) -> Option<&IndexElement> {
        self.elements.get(0)
    }
    pub fn next(&self, cur: &str) -> Option<&IndexElement> {
        let idx = self.elements.iter().position(|i| i.id == cur)?;
        self.elements.get(idx + 1)
    }
    pub fn prev(&self, cur: &str) -> Option<&IndexElement> {
        let idx = self.elements.iter().position(|i| i.id == cur)?;
        if idx > 0 {
            self.elements.get(idx - 1)
        } else {
            return None;
        }
    }
}

#[derive(Debug, Default)]
pub struct IndexElement {
    id: String,
    path: String,
}
impl IndexElement {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn path(&self) -> &str {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use crate::epub::index::{Index, IndexElement};

    fn elements() -> Vec<IndexElement> {
        vec![
            IndexElement {
                id: "aaa".to_owned(),
                path: "".to_owned(),
            },
            IndexElement {
                id: "bbb".to_owned(),
                path: "".to_owned(),
            },
            IndexElement {
                id: "ccc".to_owned(),
                path: "".to_owned(),
            },
        ]
    }

    #[test]
    fn first_item_with_elements() {
        let index = Index {
            elements: elements(),
        };
        let first = index.first().unwrap();
        let first_id = "aaa";
        assert_eq!(first_id, first.id());
    }

    #[test]
    fn first_item_with_no_elements() {
        let index = Index { elements: vec![] };
        let first = index.first();
        assert!(matches!(first, None));
    }

    #[test]
    fn next_item() {
        let cur_id = "bbb";
        let next_id = "ccc";
        let index = Index {
            elements: elements(),
        };
        let next = index.next(cur_id).unwrap();
        assert_eq!(next_id, next.id());
    }

    #[test]
    fn next_item_last() {
        let cur_id = "ccc";
        let index = Index {
            elements: elements(),
        };
        let next = index.next(cur_id);
        assert!(next.is_none());
    }

    #[test]
    fn prev_item() {
        let cur_id = "bbb";
        let prev_id = "aaa";
        let index = Index {
            elements: elements(),
        };
        let prev = index.prev(cur_id).unwrap();
        assert_eq!(prev_id, prev.id());
    }

    #[test]
    fn prev_item_first() {
        let cur_id = "aaa";
        let index = Index {
            elements: elements(),
        };
        let prev = index.prev(cur_id);
        assert!(prev.is_none());
    }
}
