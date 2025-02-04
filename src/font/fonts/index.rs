use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::{self, ReadDir};
use std::path::{Path, PathBuf};

use ttf_parser::PlatformId;

use super::face::Face;
use super::family::Family;
use super::style::FontStyle;

pub struct IndexedFont {
    pub path: PathBuf,
    pub bytes: Vec<u8>,
    pub family: String,
    pub style: FontStyle,
}
impl IndexedFont {
    pub fn new<P>(entry: P) -> Self
    where
        P: AsRef<Path>,
    {
        let bytes = std::fs::read(&entry).unwrap();
        let (family, subfamily) = read_font_metadata(&bytes);
        Self {
            path: entry.as_ref().to_owned(),
            bytes,
            family,
            style: subfamily,
        }
    }
}

// pub trait Indexer {
//     fn get_family(&self, family: &str) -> Option<Family>;
//     fn families(&self) -> impl Iterator<Item = &str>;
// }

pub struct FontIndexer {
    fonts: HashMap<String, Vec<IndexedFont>>,
}
impl FontIndexer {
    pub fn new(path: &str) -> Self {
        let mut fonts: HashMap<String, Vec<IndexedFont>> = HashMap::new();
        let scanner = IndexScanner::scan(path);
        scanner.for_each(|face| {
            let fc = face.family.clone();
            match fonts.entry(fc.clone()) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(face);
                }
                Entry::Vacant(e) => {
                    e.insert(vec![face]);
                }
            }
        });
        Self { fonts }
    }

    pub fn get_family(&self, family: &str) -> Option<Family> {
        self.fonts.get(family).map(|v| {
            let faces = v.iter().map(Face::new).collect();
            Family {
                name: family.to_owned(),
                faces,
            }
        })
    }
    pub fn families(&self) -> impl Iterator<Item = &str> {
        std::iter::empty()
    }
}
struct IndexScanner {
    dirs: Vec<PathBuf>,
    files: Option<ReadDir>,
}
impl IndexScanner {
    fn scan(path: &str) -> Self {
        Self {
            dirs: vec![PathBuf::from(path)],
            files: None,
        }
    }
}
impl Iterator for IndexScanner {
    type Item = IndexedFont;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.files.is_none() && self.dirs.is_empty() {
                return None;
            }
            while let Some(dir_files) = &mut self.files {
                match dir_files.next() {
                    Some(Ok(entry)) => {
                        let ep = entry.path();
                        if ep.is_dir() {
                            self.dirs.push(ep);
                            continue;
                        }
                        return Some(IndexedFont::new(entry.path()));
                    }
                    None => {
                        self.files = None;
                        break;
                    }
                    _ => (),
                }
            }
            while let Some(dir) = self.dirs.pop() {
                if let Ok(files) = fs::read_dir(&dir) {
                    self.files = Some(files);
                }
            }
        }
    }
}

fn read_font_metadata(data: &[u8]) -> (String, FontStyle) {
    let mut family = String::new();
    let mut subfamily = FontStyle::Regular;
    let face = ttf_parser::Face::parse(data, 0).unwrap();
    for name in face.names() {
        if name.platform_id == PlatformId::Windows {
            match name.name_id {
                1 => {
                    family = read_utf16_string(name.name);
                }
                2 => {
                    let subfam = read_utf16_string(name.name);
                    subfamily = FontStyle::from(subfam.as_str());
                }
                _ => {}
            }
        }
    }
    (family, subfamily)
}

fn read_utf16_string(raw: &[u8]) -> String {
    let v: Vec<_> = raw
        .chunks(2)
        .map(|e| {
            let a: [u8; 2] = [e[0], e[1]];
            u16::from_be_bytes(a)
        })
        .collect();
    String::from_utf16(&v).unwrap()
}
