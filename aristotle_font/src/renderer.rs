use crate::builder::Builder;
use crate::fonts::{Faces, Family, FontStyle};
use crate::geom::{Point, Rect};
use crate::{Error, Glyph, RenderingConfig, TextObject, TypesetObject};
use ttf_parser::{Face, GlyphId};

pub struct TextRenderer {
    point_size: f32,
    pub font: Option<Family>,
    pub canvas_width: u32,
    canvas_height: u32,
}
impl TextRenderer {
    pub fn new(config: &RenderingConfig) -> Self {
        Self {
            font: config.font.clone(),
            point_size: config.point_size,
            canvas_width: config.width,
            canvas_height: config.height,
        }
    }

    fn face(&self) {
        //
    }

    //pub fn config(&self) -> RenderingConfig {
    //    RenderingConfig {
    //        point_size: self.base_point_size,
    //        width: self.canvas_width,
    //        height: self.canvas_height,
    //        font_path: self.font_path.clone(),
    //    }
    //}

    pub fn update_config(&mut self, config: &RenderingConfig) {
        self.point_size = config.point_size;
        self.canvas_width = config.width;
        self.canvas_height = config.height;
    }

    pub fn set_buffer_size(&mut self, width: u32, height: u32) {
        self.canvas_width = width;
        self.canvas_height = height;
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
        return 0.0;
    }

    pub fn typeset(&mut self, text: &TextObject, pos: Point) -> Result<TypesetObject, Error> {
        // TODO: make this cleaner
        let family = self.font.as_ref().unwrap();
        let style = text.style.unwrap_or(FontStyle::Regular);
        let face = family.get_face(style).unwrap();
        let scale_factor = face.scale_factor(text.size.unwrap_or(self.point_size));
        let face = face.as_face();

        let mut last = None;
        let mut caret = pos;
        let scaled_height = face.height() as f32 * scale_factor;
        let mut glyphs = vec![];
        for c in text.raw_text.chars() {
            let gido = face.glyph_index(c);
            if gido.is_none() {
                continue;
            }
            let gid = gido.unwrap();
            let hadv = face.glyph_hor_advance(gid).unwrap() as f32 * scale_factor;
            //- face.glyph_hor_side_bearing(gid).unwrap() as f32 * self.scale_factor;
            if caret.x + hadv >= self.canvas_width as f32 {
                caret = Point::new(0.0, caret.y + scaled_height);
            }
            if caret.y + scaled_height > self.canvas_height as f32 {
                return Ok(TypesetObject::new(glyphs, pos, caret));
            }
            let pos = caret;
            caret.x += hadv;

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

            glyphs.push(Glyph { gid, pos, dim })
        }
        return Ok(TypesetObject {
            glyphs,
            start: pos,
            caret,
            size: text.size,
            style: text.style,
            ..Default::default()
        });
    }

    pub fn raster<F>(&self, text: &TypesetObject, mut pix_func: F) -> Result<(), Error>
    where
        F: FnMut(u32, u32, u8),
    {
        let family = self.font.as_ref().unwrap();
        let style = text.style.unwrap_or(FontStyle::Regular);
        let face = family.get_face(style).unwrap();
        let scale_factor = face.scale_factor(text.size.unwrap_or(self.point_size));

        let face = face.as_face();

        let mut builder = Builder::new(face.descender(), scale_factor);
        for g in text.glyphs.iter() {
            let min = g.dim.min;
            let max = g.dim.max;
            let w = (2.0 * max.x * scale_factor).ceil();
            let h = ((max.y - min.y) * scale_factor).ceil();
            builder.reset(w, h);
            if let Some(og) = face.outline_glyph(g.gid, &mut builder) {
                builder.rasteriser.for_each_pixel_2d(|x, y, v| {
                    //convert 0-1 range to 0-255
                    let mut byte = (v.clamp(0.0, 1.0) * 255.0) as u8;

                    //if there's no coverage just stop immediately
                    if !cfg!(debug_assertions) {
                        if byte == 0 {
                            return;
                        }
                    }

                    let bbox_min_x = og.x_min as f32 * scale_factor;
                    let bbox_max_x = og.x_max as f32 * scale_factor;
                    let bbox_min_y = (og.y_min as f32 - min.y) * scale_factor;
                    let bbox_max_y = (og.y_max as f32 - min.y) * scale_factor;

                    // don't draw pixels we know are outside the bbox
                    if x < bbox_min_x as u32
                        || x > bbox_max_x as u32
                        || y > bbox_max_y as u32
                        || y < bbox_min_y as u32
                    {
                        return;
                    }

                    // invert so that more coverage means less fill
                    byte = 255 - byte;

                    // draw the bbox
                    if cfg!(debug_assertions) {
                        if x == (og.x_min as f32 * scale_factor) as u32
                            || x == (og.x_max as f32 * scale_factor) as u32
                            || y == ((og.y_min as f32 - min.y) as f32 * scale_factor) as u32
                            || y == ((og.y_max as f32 - min.y) as f32 * scale_factor) as u32
                        {
                            byte = 0;
                        }
                    }

                    // don't draw white pixels inside the bbox either
                    if byte == 255 {
                        return;
                    }

                    // invert glyph along the y-axis
                    let y = h as u32 - y;

                    // translate xy coords to the glyph position
                    let x = x + g.pos.x as u32;
                    let y = y + g.pos.y as u32;

                    pix_func(x, y, byte);
                });
            }
        }
        Ok(())
    }
}
