use ttf_parser::{Face, GlyphId};

use crate::fonts::{Faces, Family, FontStyle};
use crate::geom::{Point, Rect};
use crate::raster::raster;
use crate::{ContentElement, Error, FontWeight, Glyph, TypesetConfig};

#[derive(Clone, Default, Debug)]
pub struct TypesetElement {
    pub caret: Point,
    pub glyphs: Vec<Glyph>,
    pub point_size: f32,
    pub style: FontStyle,
    pub weight: Option<FontWeight>,
}
impl TypesetElement {
    pub fn new(glyphs: Vec<Glyph>, caret: Point, point_size: f32, style: FontStyle) -> Self {
        Self {
            glyphs,
            caret,
            point_size,
            style,
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub struct Typesetter {
    family: Family,
    scaled_height: f32,
    caret: Point,
    elements: Vec<TypesetElement>,
    params: TypesetConfig,
}

impl Typesetter {
    pub fn new(config: TypesetConfig, family: Family) -> Result<Self, Error> {
        let font = family
            .get_face(FontStyle::Regular)
            .ok_or(Error::MissingFace)?;
        let face = font.as_face();
        let scale_factor = font.scale_factor(config.point_size);
        let scaled_height = face.height() as f32 * scale_factor;
        Ok(Self {
            family,
            scaled_height,
            params: config,
            caret: Point::default(),
            elements: vec![],
        })
    }

    pub fn set_buffer_size(&mut self, width: u32, height: u32) {
        self.params.page_width = width;
        self.params.page_height = height;
    }

    pub fn clear(&mut self) {
        self.caret = Point::default();
        self.elements.clear()
    }

    pub fn raster<F>(&self, mut f: F) -> Result<(), Error>
    where
        F: FnMut(u32, u32, u8),
    {
        self.elements
            .iter()
            .try_for_each(|e| raster(&self.family, e, &mut f))
    }

    pub fn typeset(&mut self, text: &ContentElement) -> Result<(), Error> {
        match text {
            ContentElement::Linebreak => {
                if self.overflows_vertically() {
                    return Err(Error::PageOverflow);
                }
                self.caret = Point::new(
                    self.params.horizontal_margin,
                    self.caret.y + self.scaled_height,
                );
                Ok(())
            }
            ContentElement::Paragraph => {
                if self.overflows_vertically() {
                    return Err(Error::PageOverflow);
                }
                let hadv = self.params.page_width as f32 * 0.04;
                self.caret = Point::new(
                    self.params.horizontal_margin + hadv,
                    self.caret.y + self.scaled_height,
                );
                //self.caret = Point::new(self.caret.x + hadv, self.caret.y);
                //self.caret = Point::new(hadv, self.caret.y);
                Ok(())
            }
            ContentElement::Text(text) => {
                let style = text.style.unwrap_or(FontStyle::Regular);
                let font = self.family.get_face(style).ok_or(Error::MissingFace)?;
                let point_size = text.size.unwrap_or(self.params.point_size);
                let scale_factor = font.scale_factor(point_size);
                let face = font.as_face();

                let mut last = None;
                let mut glyphs = vec![];
                let start_caret = self.caret;
                for (i, c) in text.raw_text.chars().enumerate() {
                    let gido = face.glyph_index(c);
                    if gido.is_none() {
                        continue;
                    }
                    let gid = gido.unwrap();
                    let hadv = face.glyph_hor_advance(gid).unwrap() as f32 * scale_factor;
                    //- face.glyph_hor_side_bearing(gid).unwrap() as f32 * scale_factor;
                    // if this char would overextend horizontally
                    if self.overflows_horizontally(hadv) {
                        // if it would then overextend vertically
                        if self.overflows_vertically() {
                            //return the current char index
                            return Err(Error::ContentOverflow(i));
                        }
                        self.caret = Point::new(
                            self.params.horizontal_margin,
                            self.caret.y + self.scaled_height,
                        );
                    }
                    let prev_caret = self.caret;
                    self.caret.x += hadv;

                    Self::kern(&face, last, gid);
                    last = Some(gid);

                    let x_min = face.glyph_hor_side_bearing(gid).unwrap() as f32;
                    let x_max = face.glyph_hor_advance(gid).unwrap() as f32;
                    let y_min = face.descender() as f32;
                    let y_max = face.ascender() as f32;
                    let dim = Rect {
                        min: Point::new(x_min, y_min),
                        max: Point::new(x_max, y_max),
                    };

                    glyphs.push(Glyph {
                        gid,
                        pos: prev_caret,
                        dim,
                    })
                }
                let t = TypesetElement {
                    glyphs,
                    point_size,
                    style,
                    caret: start_caret,
                    ..Default::default()
                };
                self.elements.push(t);
                Ok(())
            }
        }
    }

    fn kern(face: &Face, left: Option<GlyphId>, right: GlyphId) -> f32 {
        let mut h_kern: Vec<_> = face
            .tables()
            .kern
            .iter()
            .flat_map(|c| c.subtables)
            .filter(|st| st.horizontal && !st.variable)
            .collect();
        if let Some(last_gid) = left {
            let kern_val = h_kern
                .iter_mut()
                .find_map(|i| i.glyphs_kerning(last_gid, right))
                .map(f32::from)
                .unwrap_or_default();
            if kern_val != 0.0 {
                dbg!(last_gid, right, kern_val);
            }
        }
        0.0
    }

    fn overflows_horizontally(&self, hadv: f32) -> bool {
        self.caret.x + hadv > self.params.page_width as f32 - self.params.horizontal_margin
    }

    fn overflows_vertically(&self) -> bool {
        self.caret.y + self.scaled_height
            > self.params.page_height as f32 - self.params.vertical_margin
    }
}
