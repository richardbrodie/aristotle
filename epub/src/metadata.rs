#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Metadata {
    title: String,
    language: String,
    identifier: String,
    author: Option<String>,
    published: Option<String>,
}
impl Metadata {
    pub fn new(
        title: String,
        language: String,
        identifier: String,
        author: Option<String>,
        published: Option<String>,
    ) -> Self {
        Self {
            title,
            language,
            identifier,
            author,
            published,
        }
    }
}
