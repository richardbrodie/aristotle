use std::ops::DerefMut;

use crate::text::fonts::Family;
use crate::text::typeset::TypesetText;
use crate::text::FontError;

use super::builder::Builder;

pub fn text<B>(
    family: &Family,
    text: &TypesetText,
    width: usize,
    buffer: &mut B,
) -> Result<(), FontError>
where
    B: DerefMut<Target = [u32]>,
{
    let face = family.face(text.style)?;
    let scale_factor = face.scale_factor(text.point_size);
    let face = face.as_ttf_face()?;

    let desc = face.descender() as f32;
    let asc = face.ascender() as f32;
    let h = ((asc - desc) * scale_factor).ceil();

    let mut builder = Builder::new(face.descender(), scale_factor);
    for g in text.glyphs.iter() {
        let w = (g.advance * scale_factor).ceil();
        builder.reset(w as usize, h as usize, -g.bearing);
        if let Some(og) = face.outline_glyph(g.id, &mut builder) {
            builder.rasteriser.for_each_pixel_2d(|x, y, v| {
                //convert 0-1 range to 0-255
                let mut byte = (v.clamp(0.0, 1.0) * 255.0) as u8;

                //if there's no coverage just stop immediately
                if byte == 0 {
                    return;
                }

                let bbox_min_x = (og.x_min as f32 - g.bearing) * scale_factor;
                let bbox_max_x = (og.x_max as f32 - g.bearing) * scale_factor;
                let bbox_min_y = (og.y_min as f32 - g.desc) * scale_factor;
                let bbox_max_y = (og.y_max as f32 - g.desc) * scale_factor;

                // don't draw pixels we know are outside the bbox
                if x < bbox_min_x as u32
                    || x > bbox_max_x as u32
                    || y > bbox_max_y as u32
                    || y < bbox_min_y as u32
                {
                    return;
                }

                // don't draw white pixels inside the bbox either
                if byte == 0 {
                    return;
                }

                // invert so that more coverage means less fill
                byte = 255 - byte;

                // invert glyph along the y-axis
                let y = h as u32 - y;

                // translate xy coords to the glyph position
                //let xoff = (og.x_min as f32 + min.x) * scale_factor;
                let xoff = og.x_min as f32 * scale_factor;
                //let xoff = min.x as f32 * scale_factor;
                let x = x + (g.pos.x + xoff) as u32;
                let y = y + g.pos.y as u32;

                let z = byte as u32;
                let c = z | z << 8 | z << 16;
                let idx = x as usize + y as usize * width;
                buffer[idx] = c;
            });
        }
    }
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use ttf_parser::GlyphId;

    use crate::text::fonts::{Family, FontStyle};
    use crate::text::geom::{Point, Rect};
    use crate::text::typeset::TypesetText;
    use crate::text::Glyph;

    use super::text;

    pub fn test_family(style: FontStyle) -> Family {
        let path = match style {
            FontStyle::Regular => "testfiles/fonts/vollkorn/Vollkorn-Regular.otf",
            FontStyle::Italic => "testfiles/fonts/vollkorn/Vollkorn-Italic.ttf",
            FontStyle::Bold => "testfiles/fonts/vollkorn/Vollkorn-Bold.ttf",
            FontStyle::BoldItalic => "testfiles/fonts/vollkorn/Vollkorn-BoldItalic.otf",
            FontStyle::Mono => "",
        };
        Family::from_font(path)
    }

    #[test]
    fn regular_y() {
        let p = Point {
            x: 103.248,
            y: 33.432,
        };
        let w = 640;
        let t = TypesetText {
            glyphs: vec![Glyph {
                id: GlyphId(588),
                pos: p,
                bearing: -118.0,
                advance: -441.0,
                desc: 453.0,
            }],
            point_size: 18.0,
            style: FontStyle::Regular,
        };
        let fam = test_family(t.style);
        let mut buf = vec![0; p.x as usize * w];
        let r = text(&fam, &t, w, &mut buf);
        assert!(r.is_ok());
    }

    #[test]
    fn italic_y() {
        let p = Point {
            x: 103.248,
            y: 33.432,
        };
        let w = 640;
        let t = TypesetText {
            glyphs: vec![
                //},
                Glyph {
                    id: GlyphId(588),
                    pos: p,
                    bearing: -118.0,
                    advance: -441.0,
                    desc: 453.0,
                },
            ],
            point_size: 18.0,
            style: FontStyle::Italic,
        };
        let fam = test_family(t.style);
        let mut buf = vec![0; p.x as usize * w];
        let r = text(&fam, &t, w, &mut buf);
        assert!(r.is_ok());
    }
}
