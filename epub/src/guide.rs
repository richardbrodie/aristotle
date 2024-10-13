use std::path::PathBuf;

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Reference {
    ref_type: String,
    title: String,
    href: PathBuf,
}
impl Reference {
    pub fn new(ref_type: String, title: String, href: String) -> Self {
        Self {
            ref_type,
            title,
            href: PathBuf::from(href),
        }
    }
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Guide {
    references: Vec<Reference>,
}
impl Guide {
    pub fn new(references: Vec<Reference>) -> Self {
        Self { references }
    }
}
