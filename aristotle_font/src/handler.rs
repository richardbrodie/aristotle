use crate::builder::Builder;
use crate::geom::{Point, Rect};
use ttf_parser::{Face, GlyphId};

pub struct Glyph {
    gid: GlyphId,
    pos: Point,
    dim: Rect,
}

fn scale_factor(font_size: f32, font: &Face) -> f32 {
    let px_per_em = font_size * (96.0 / 72.0);
    let units_per_em = font.units_per_em() as f32;
    px_per_em / units_per_em
}

pub struct GlyphHandler {
    font_data: Vec<u8>,
    pub width: u32,
    height: u32,
    pub font_size: f32,
    scale_factor: f32,
    raw_text: String,
    typeset_text: Vec<Glyph>,
}
impl GlyphHandler {
    pub fn new(path: &str) -> Self {
        let font_path = path;
        let font_data = std::fs::read(font_path).unwrap();
        let face = Face::parse(&font_data, 0).unwrap();
        dbg!(face.line_gap());
        dbg!(face.width());

        let font_size = 24.0;
        let scale_factor = scale_factor(font_size, &face);

        Self {
            font_data,
            width: 0,
            height: 0,
            font_size,
            scale_factor,
            raw_text: String::new(),
            typeset_text: vec![],
        }
    }

    //fn load_font(&mut self) {
    //    println!("loading font {}", self.font_idx);
    //    let font_path = FONT_PATHS[self.font_idx];
    //    self.font_data = std::fs::read(font_path).unwrap();
    //    self.typeset();
    //}

    pub fn get_face(&self) -> Face {
        let face = Face::parse(&self.font_data, 0).unwrap();
        return face;
    }

    pub fn set_buffer_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.typeset();
    }
    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
        let face = Face::parse(&self.font_data, 0).unwrap();
        self.scale_factor = scale_factor(size, &face);
        self.typeset();
    }

    pub fn set_text(&mut self, text: &str) {
        self.raw_text.push_str(text);
        self.typeset();
    }

    pub fn clear_text(&mut self) {
        self.raw_text.truncate(0);
        self.typeset_text.truncate(0);
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

    fn typeset(&mut self) {
        let face = Face::parse(&self.font_data, 0).unwrap();
        self.typeset_text.truncate(0);
        let mut last = None;
        let mut caret = Point::default();
        let scaled_height = face.height() as f32 * self.scale_factor;
        for c in self.raw_text.chars() {
            let gido = face.glyph_index(c);
            if gido.is_none() {
                continue;
            }
            let gid = gido.unwrap();
            let hadv = face.glyph_hor_advance(gid).unwrap() as f32 * self.scale_factor;
            //- face.glyph_hor_side_bearing(gid).unwrap() as f32 * self.scale_factor;
            if caret.x + hadv >= self.width as f32 {
                caret = Point::new(0.0, caret.y + scaled_height);
            }
            if caret.y + scaled_height > self.height as f32 {
                return;
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

            self.typeset_text.push(Glyph { gid, pos, dim })
        }
    }

    pub fn raster<F>(&self, mut pix_func: F)
    where
        F: FnMut(u32, u32, u8),
    {
        let font = self.get_face();
        let mut builder = Builder::new(font.descender(), self.scale_factor);
        for g in self.typeset_text.iter() {
            let min = g.dim.min;
            let max = g.dim.max;
            let w = (2.0 * max.x * self.scale_factor).ceil();
            let h = ((max.y - min.y) * self.scale_factor).ceil();
            builder.reset(w, h);
            if let Some(og) = font.outline_glyph(g.gid, &mut builder) {
                builder.rasteriser.for_each_pixel_2d(|x, y, v| {
                    //convert 0-1 range to 0-255
                    let mut byte = (v.clamp(0.0, 1.0) * 255.0) as u8;

                    //if there's no coverage just stop immediately
                    if !cfg!(debug_assertions) {
                        if byte == 0 {
                            return;
                        }
                    }

                    let bbox_min_x = og.x_min as f32 * self.scale_factor;
                    let bbox_max_x = og.x_max as f32 * self.scale_factor;
                    let bbox_min_y = (og.y_min as f32 - min.y) * self.scale_factor;
                    let bbox_max_y = (og.y_max as f32 - min.y) * self.scale_factor;

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
                        if x == (og.x_min as f32 * self.scale_factor) as u32
                            || x == (og.x_max as f32 * self.scale_factor) as u32
                            || y == ((og.y_min as f32 - min.y) as f32 * self.scale_factor) as u32
                            || y == ((og.y_max as f32 - min.y) as f32 * self.scale_factor) as u32
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
    }
}
