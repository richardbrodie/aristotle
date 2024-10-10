use std::fs::File;
use std::io::{Read, Seek};
use std::path::{Path, PathBuf};
use zip::ZipArchive;

use crate::document::Document;
use crate::guide::{Guide, Reference};
use crate::manifest::{Manifest, ManifestItem};
use crate::metadata::Metadata;
use crate::spine::Spine;
use crate::Error;

pub struct Book {
    sourcefile: ZipArchive<File>,
    contents_dir: PathBuf,
    metadata: Metadata,
    manifest: Manifest,
    spine: Spine,
    guide: Option<Guide>,
}
impl Book {
    pub fn new(p: &Path) -> Result<Self, Error> {
        // open epub file
        let epub_file = File::open(p).unwrap();
        let mut epub = ZipArchive::new(epub_file).unwrap();

        // read META-INF
        let rootfile = find_rootfile(&mut epub)?;
        let contents_dir = rootfile.parent().unwrap().to_owned();

        // parse the contents.opf
        let mut file_bytes = Vec::new();
        let mut contents_opf = epub
            .by_name(rootfile.to_str().unwrap())
            .map_err(|_| Error::Zip)?;
        let _ = contents_opf.read_to_end(&mut file_bytes).unwrap();
        let file_contents = std::str::from_utf8(&file_bytes).unwrap();
        let doc = roxmltree::Document::parse(file_contents).unwrap();

        let metadata = extract_metadata(&doc)?;
        let manifest = extract_manifest(&doc)?;
        let spine = extract_spine(&doc)?;
        let guide = extract_guide(&doc)?;
        drop(contents_opf);

        return Ok(Book {
            sourcefile: epub,
            contents_dir,
            metadata,
            manifest,
            spine,
            guide,
        });
    }
    pub fn items(&self) -> impl Iterator<Item = Document> + '_ {
        self.spine
            .items()
            .map_while(move |id| self.manifest.find(id))
            .map(Into::into)
    }
    pub fn next_item(&self, id: &str) -> Option<Document> {
        self.spine
            .next(id)
            .and_then(|i| self.manifest.find(i))
            .map(Into::into)
    }
}

fn extract_guide<'a>(doc: &roxmltree::Document<'a>) -> Result<Option<Guide>, Error> {
    let guide_node = doc.descendants().find(|n| n.has_tag_name("guide"));
    if guide_node.is_none() {
        return Ok(None);
    }

    let references: Vec<_> = guide_node
        .unwrap()
        .children()
        .into_iter()
        .filter(|node| node.has_tag_name("itemref"))
        .map(|node| {
            let mut href = String::new();
            let mut ref_type = String::new();
            let mut title = String::new();
            for attr in node.attributes() {
                match attr.name() {
                    "href" => href = attr.value().to_owned(),
                    "title" => title = attr.value().to_owned(),
                    "type" => ref_type = attr.value().to_owned(),
                    _ => {}
                }
            }
            Reference::new(ref_type, title, href)
        })
        .collect();
    return Ok(Some(Guide::new(references)));
}

fn extract_spine<'a>(doc: &roxmltree::Document<'a>) -> Result<Spine, Error> {
    let spine_node = doc.descendants().find(|n| n.has_tag_name("spine")).unwrap();

    let toc = spine_node
        .attributes()
        .find(|attr| attr.name() == "toc")
        .map(|a| a.value().to_owned())
        .unwrap();
    let itemrefs: Vec<_> = spine_node
        .children()
        .into_iter()
        .filter_map(|node| match node.has_tag_name("itemref") {
            true => node
                .attributes()
                .find(|a| a.name() == "idref")
                .map(|a| a.value().to_owned()),
            false => None,
        })
        .collect();
    return Ok(Spine::new(toc, itemrefs));
}

fn extract_manifest<'a>(doc: &roxmltree::Document<'a>) -> Result<Manifest, Error> {
    let manifest_node = doc
        .descendants()
        .find(|n| n.has_tag_name("manifest"))
        .unwrap();

    let items: Vec<_> = manifest_node
        .children()
        .into_iter()
        .filter(|node| node.has_tag_name("item"))
        .map(|node| {
            let mut href = String::new();
            let mut mediatype = String::new();
            let mut id = String::new();
            for attr in node.attributes() {
                match attr.name() {
                    "href" => href = attr.value().to_owned(),
                    "id" => id = attr.value().to_owned(),
                    "media-type" => mediatype = attr.value().to_owned(),
                    _ => {}
                }
            }
            ManifestItem::new(mediatype, id, href)
        })
        .collect();
    return Ok(Manifest::new(items));
}

fn extract_metadata<'a>(doc: &roxmltree::Document<'a>) -> Result<Metadata, Error> {
    let meta_node = doc
        .descendants()
        .find(|n| n.has_tag_name("metadata"))
        .unwrap();

    let mut title = String::new();
    let mut language = String::new();
    let mut identifier = String::new();
    let mut author = None;
    let mut published = None;
    for node in meta_node.children() {
        if node.tag_name().namespace() == Some("http://purl.org/dc/elements/1.1/") {
            match node.tag_name().name() {
                "title" => title = node.text().unwrap().to_owned(),
                "language" => language = node.text().unwrap().to_owned(),
                "identifier" => identifier = node.text().unwrap().to_owned(),
                "creator" => author = node.text().map(ToOwned::to_owned),
                "date" => published = node.text().map(ToOwned::to_owned),
                _ => {}
            }
        }
    }
    let metadata = Metadata::new(title, language, identifier, author, published);
    return Ok(metadata);
}

fn find_rootfile<R>(epub_file: &mut ZipArchive<R>) -> Result<PathBuf, Error>
where
    R: Read + Seek,
{
    let mut file_bytes = Vec::new();
    {
        let mut container = epub_file.by_name("META-INF/container.xml").unwrap();
        let _ = container.read_to_end(&mut file_bytes).unwrap();
    }
    let file_contents = std::str::from_utf8(&file_bytes).unwrap();

    let doc = roxmltree::Document::parse(file_contents).unwrap();
    let rootfile_path = doc
        .descendants()
        .find(|n| n.has_tag_name("rootfile"))
        .and_then(|elem| {
            elem.attributes()
                .find(|n| n.name() == "full-path")
                .map(|a| a.value())
        })
        .map(|s| PathBuf::from(s))
        .ok_or_else(|| Error::Xml);
    rootfile_path
}
