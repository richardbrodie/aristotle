use ttf_parser::{Face, GlyphId};

use super::fonts::FontStyle;
use super::geom::{Point, Rect};
use super::{FontError, Glyph, TextObject, TypesetConfig};

#[derive(Debug, Default)]
pub struct Text {
    pub glyphs: Vec<Glyph>,
    pub point_size: f32,
    pub style: FontStyle,
}
#[derive(Debug)]
pub struct Element {
    pub caret: Point,
    pub text: Text,
}

#[derive(Debug)]
pub struct Typesetter {
    scaled_height: f32,
    space_width: f32,
    params: TypesetConfig,
    // family: Family,
}

pub enum TResult<'a> {
    Ok(Element),
    Overflow {
        processed: Element,
        remainder: &'a str,
    },
    Error(FontError),
}
//impl From<Error> for TResult {
//    fn from(e: Error) -> Self {
//        TResult::Error(e)
//    }
//}

impl Typesetter {
    pub fn new(config: &TypesetConfig) -> Result<Self, FontError> {
        let face = config.family.face(FontStyle::default())?;
        let scaled_height = face.scaled_height(config.point_size)?;
        let advance = face.space_width(config.point_size)?;
        tracing::info!(
            "initial page dims: {}x{}",
            config.page_width,
            config.page_height
        );
        Ok(Self {
            scaled_height,
            space_width: advance,
            params: config.to_owned(),
            // family,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        tracing::info!("new page dims: {}x{}", width, height);
        self.params.page_width = width;
        self.params.page_height = height;
    }

    pub fn new_caret(&self) -> Point {
        Point::new(
            self.params.horizontal_margin.into(),
            self.params.vertical_margin.into(),
        )
    }

    pub fn line_height(&self) -> f32 {
        self.scaled_height
    }

    pub fn linebreak(&self, caret: Point) -> Result<Point, FontError> {
        if self.overflows_vertically(caret) {
            return Err(FontError::PageOverflow);
        }
        let new_caret = Point::new(
            self.params.horizontal_margin.into(),
            caret.y + self.scaled_height,
        );
        Ok(new_caret)
    }

    pub fn paragraph(&self, caret: Point) -> Result<Point, FontError> {
        if self.overflows_vertically(caret) {
            return Err(FontError::PageOverflow);
        }
        let hadv = self.params.page_width as f32 * 0.03;
        let new_caret = Point::new(
            self.params.horizontal_margin as f32 + hadv,
            caret.y + self.scaled_height,
        );
        Ok(new_caret)
    }

    pub fn heading<'a>(&'a self, caret: Point, t: &'a TextObject) -> TResult<'a> {
        let style = FontStyle::Bold;
        let size = t.size.unwrap_or(self.params.point_size);
        self.typeset(caret, &t.raw_text, size, style)
    }

    pub fn text<'a>(&'a self, caret: Point, t: &'a TextObject) -> TResult<'a> {
        let style = t.style.unwrap_or(FontStyle::Regular);
        let size = t.size.unwrap_or(self.params.point_size);
        self.typeset(caret, &t.raw_text, size, style)
    }

    fn typeset<'a>(
        &'a self,
        caret: Point,
        text: &'a str,
        size: f32,
        style: FontStyle,
    ) -> TResult<'a> {
        let font = self.params.family.face(style).unwrap();
        let scale_factor = font.scale_factor(size);
        let face = font.as_ttf_face().unwrap();

        // face metrics
        let desc = face.descender() as f32;
        let asc = face.ascender() as f32;

        //let mut last = None;
        let mut word_buffer: Vec<Glyph> = vec![];
        let mut char_buf = vec![];
        let mut last_word = String::new();
        let mut caret = caret;
        let mut last_committed_character = 0;
        let mut count = 0;
        let mut t = Text {
            glyphs: vec![],
            point_size: size,
            style,
        };
        for (i, c) in text.chars().enumerate() {
            if c.is_whitespace() {
                count += word_buffer.len() + 1;
                t.glyphs.append(&mut word_buffer);
                caret.x += self.space_width;
                last_committed_character = i;
                last_word = String::from_iter(char_buf);
                char_buf = vec![];
                continue;
            }
            let gid = face.glyph_index(c).ok_or(FontError::NoGlyph(c)).unwrap();

            // char metrics
            let bearing = face.glyph_hor_side_bearing(gid).unwrap() as f32;
            let advance = face.glyph_hor_advance(gid).unwrap() as f32;

            let hadv = advance * scale_factor;
            let _offset = bearing * scale_factor;

            // if this char would overextend horizontally
            if self.overflows_horizontally(caret, hadv) {
                // if it would then overextend vertically
                if self.overflows_vertically(caret) {
                    //return the current char index
                    tracing::info!(
                        "word: {}, buf: {:?}, char: {}, idx: {}/{}/{}",
                        last_word,
                        char_buf,
                        c,
                        count,
                        last_committed_character,
                        i
                    );
                    let e = Element { caret, text: t };
                    let res = &text[last_committed_character..];
                    return TResult::Overflow {
                        processed: e,
                        remainder: res,
                    };
                }
                caret = Point::new(
                    self.params.horizontal_margin.into(),
                    caret.y + self.scaled_height,
                );
                for g in word_buffer.iter_mut() {
                    g.pos = caret;
                    caret.x += g.dim.max.x * scale_factor;
                }
            }
            let prev_caret = caret;
            caret.x += hadv;

            //Self::kern(&face, last, gid);
            //last = Some(gid);

            let dim = Rect {
                min: Point::new(bearing, desc),
                max: Point::new(advance, asc),
            };

            let pos = Point::new(prev_caret.x, prev_caret.y);
            word_buffer.push(Glyph { gid, pos, dim });
            char_buf.push(c);
        }

        // add last word
        t.glyphs.append(&mut word_buffer);

        TResult::Ok(Element { caret, text: t })
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

    fn overflows_horizontally(&self, caret: Point, hadv: f32) -> bool {
        caret.x + hadv + self.params.horizontal_margin as f32 > self.params.page_width as f32
    }

    fn overflows_vertically(&self, caret: Point) -> bool {
        caret.y + self.scaled_height + self.params.vertical_margin as f32
            >= self.params.page_height as f32
    }
}
