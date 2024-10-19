use quick_xml::events::Event;
use quick_xml::Reader;
use std::usize;

#[derive(Debug, Default)]
pub struct Content {
    raw_content: Vec<u8>,
}
impl Content {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { raw_content: bytes }
    }
    pub fn content(&self) -> Option<&str> {
        let str_content = std::str::from_utf8(&self.raw_content).unwrap();
        let mut reader = Reader::from_str(str_content);
        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"body" => {
                    let c = reader.read_to_end(e.name()).unwrap();
                    let start = c.start as usize;
                    let end = c.end as usize;
                    return Some(&str_content[start..end]);
                }
                Err(_) => return None,
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                _ => (),
            }
        }
        return None;
    }
}
