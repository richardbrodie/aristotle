#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum FontStyle {
    #[default]
    Regular,
    Bold,
    Italic,
    BoldItalic,
    Mono,
}
impl From<&str> for FontStyle {
    fn from(value: &str) -> Self {
        match value {
            "Regular" => Self::Regular,
            "Bold" => Self::Bold,
            "Italic" => Self::Italic,
            "Bold Italic" => Self::BoldItalic,
            "Mono" | "Book" => Self::Mono,
            _ => Self::Regular,
        }
    }
}
