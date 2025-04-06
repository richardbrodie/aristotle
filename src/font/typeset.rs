use std::borrow::Cow;

use ttf_parser::{Face, GlyphId};

use super::caret::Caret;
use super::fonts::FontStyle;
use super::geom::{Point, Rect};
use super::{FontError, Glyph, TypesetConfig};

#[derive(Debug, Default)]
pub struct TypesetText {
    pub glyphs: Vec<Glyph>,
    pub point_size: f32,
    pub style: FontStyle,
}

pub enum TResult<'a> {
    Ok(TypesetText),
    Overflow {
        processed: TypesetText,
        remainder: Cow<'a, str>,
    },
    Error(FontError),
}
// impl From<Error> for TResult {
//     fn from(e: Error) -> Self {
//         TResult::Error(e)
//     }
// }

pub fn typeset<'a>(
    params: &TypesetConfig,
    caret: &mut Caret,
    text: &'a str,
    style: FontStyle,
) -> TResult<'a> {
    let font = params.family.face(style).unwrap();
    let scale_factor = font.scale_factor(params.point_size);
    let face = font.as_ttf_face().unwrap();

    // face metrics
    let desc = face.descender() as f32;
    let asc = face.ascender() as f32;

    let mut word_buffer: Vec<Glyph> = vec![];
    // let mut buffer_height = 0.0;
    let mut char_buf = vec![];
    let mut last_word = String::new();
    let mut last_committed_character = 0;
    let mut count = 0;
    let mut t = TypesetText {
        glyphs: vec![],
        point_size: params.point_size,
        style,
    };
    for (i, c) in text.chars().enumerate() {
        // cycle the word buffer
        if c.is_whitespace() {
            count += word_buffer.len() + 1;
            t.glyphs.append(&mut word_buffer);
            caret.space();
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
        if caret.overflows_horizontally(hadv) {
            // if it would then overextend vertically
            if caret.overflows_vertically(1.0) {
                //return the current char index
                // tracing::info!(
                //     "word: {}, buf: {:?}, char: {}, idx: {}/{}/{}",
                //     last_word,
                //     char_buf,
                //     c,
                //     count,
                //     last_committed_character,
                //     i
                // );
                let res = &text[last_committed_character..];
                return TResult::Overflow {
                    processed: t,
                    remainder: Cow::Borrowed(res),
                };
            }
            caret.newline(1.0);
            for g in word_buffer.iter_mut() {
                g.pos = caret.point();
                let gadv = g.dim.max.x * scale_factor;
                caret.advance(gadv);
            }
        }
        let pos = caret.point();
        caret.advance(hadv);

        // kern(&face, last, gid);
        // last = Some(gid);

        let dim = Rect {
            min: Point::new(bearing, desc),
            max: Point::new(advance, asc),
        };

        word_buffer.push(Glyph { gid, pos, dim });
        char_buf.push(c);
    }

    // add last word
    t.glyphs.append(&mut word_buffer);

    TResult::Ok(t)
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
