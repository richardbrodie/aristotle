use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::{self, DirEntry, ReadDir};
use std::path::PathBuf;

use ttf_parser::PlatformId;

use crate::FontStyle;

pub trait Faces {
    fn get_face(&self, style: FontStyle) -> Option<ttf_parser::Face>;
    fn styles(&self) -> impl Iterator<Item = FontStyle>;
}

pub struct Family {
    pub name: String,
    faces: Vec<Face>,
}

#[derive(Default, Clone, Debug)]
pub struct Face {
    pub bytes: Vec<u8>,
    pub filename: String,
    pub family: String,
    pub subfamily: FontStyle,
}

pub struct Indexer {
    pub faces: HashMap<String, Family>,
}

impl Indexer {
    pub fn new(path: &str) -> Self {
        let mut faces: HashMap<String, Family> = HashMap::new();
        let scanner = IndexScanner::scan(path);
        scanner.for_each(|face| {
            let fc = face.family.clone();
            match faces.entry(fc.clone()) {
                Entry::Occupied(mut e) => {
                    e.get_mut().faces.push(face);
                }
                Entry::Vacant(e) => {
                    e.insert(Family {
                        name: fc,
                        faces: vec![face],
                    });
                }
            }
        });

        Self { faces }
    }
    pub fn get_family(&self, family: &str) -> Option<&Family> {
        self.faces.get(family)
    }
    pub fn families(&self) -> impl Iterator<Item = &str> {
        self.faces.keys().map(|k| k.as_ref())
    }
}

impl Faces for Family {
    fn get_face(&self, style: FontStyle) -> Option<ttf_parser::Face> {
        self.faces
            .iter()
            .find(|s| s.subfamily == style)
            .map(|f| ttf_parser::Face::parse(&f.bytes, 0).unwrap())
    }
    fn styles(&self) -> impl Iterator<Item = FontStyle> {
        self.faces.iter().map(|f| f.subfamily)
    }
}

fn make_face(entry: &DirEntry) -> Face {
    let filename = entry.file_name().into_string().unwrap();
    let bytes = std::fs::read(entry.path()).unwrap();
    let (family, subfamily) = parse_font_names(&bytes);
    Face {
        bytes,
        family,
        filename,
        subfamily,
    }
}

fn parse_font_names(data: &[u8]) -> (String, FontStyle) {
    let mut family = String::new();
    let mut subfamily = FontStyle::Regular;
    let face = ttf_parser::Face::parse(data, 0).unwrap();
    for name in face.names() {
        if name.platform_id == PlatformId::Windows {
            match name.name_id {
                1 => {
                    family = decode_name(name.name);
                }
                2 => {
                    let subfam = decode_name(name.name);
                    subfamily = FontStyle::from(subfam.as_str());
                }
                _ => {}
            }
        }
    }
    (family, subfamily)
}

fn decode_name(raw: &[u8]) -> String {
    let v: Vec<_> = raw
        .chunks(2)
        .map(|e| {
            let a: [u8; 2] = [e[0], e[1]];
            u16::from_be_bytes(a)
        })
        .collect();
    String::from_utf16(&v).unwrap()
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
    type Item = Face;
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
                        return Some(make_face(&entry));
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
