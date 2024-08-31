use std::ops::Add;

use ab_glyph::{Font, FontVec, PxScale, ScaleFont};
use ab_glyph_rasterizer::{point, Point, Rasterizer};
use rustybuzz::ttf_parser::{GlyphId, Rect};
use rustybuzz::{Face, GlyphPosition};
use rustybuzz::{GlyphBuffer, UnicodeBuffer};
use unicode_normalization::UnicodeNormalization;

use crate::SoftBuffer;

struct Builder {
    pos: Point,
    rasteriser: ab_glyph_rasterizer::Rasterizer,
}

impl rustybuzz::ttf_parser::OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.pos = point(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p = point(x, y);
        self.rasteriser.draw_line(self.pos, p);
        self.pos = p;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let p1 = point(x1, y1);
        let p2 = point(x2, y2);
        self.rasteriser.draw_quad(self.pos, p1, p2);
        self.pos = p2;
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        let p1 = point(x1, y1);
        let p2 = point(x2, y2);
        let p3 = point(x3, y3);
        self.rasteriser.draw_cubic(self.pos, p1, p2, p3);
        self.pos = p3;
    }

    fn close(&mut self) {
        //println!("close")
        //self.rasteriser.for_each_pixel_2d()
    }
}

pub struct Glyph {
    id: GlyphId,
    pos: Point,
    bbox: Rect,
}

pub struct GlyphHandler {
    font_data: Vec<u8>,
    width: f32,
    height: f32,
    pub font_size: f32,
}

impl GlyphHandler {
    pub fn new() -> Self {
        //let font_path = "testfiles/DejaVuSansMono.ttf";
        //let font_path = "testfiles/Mido.otf";
        //let font_path = "testfiles/Vollkorn-Regular.otf";
        //let font_path = "testfiles/Exo2-Light.otf";
        //let font_path = "testfiles/OpenSans-Italic.ttf";
        //let font_path = "testfiles/SourceSansVariable-Roman.ttf";

        let font_path = "testfiles/Vollkorn-Regular.otf";
        let font_data = std::fs::read(font_path).unwrap();

        Self {
            //font,
            font_data,
            width: 0.0,
            height: 0.0,
            font_size: 24.0,
        }
    }

    //fn scale_factor(&self, font: Face, height: f32, units_per_em: f32) -> f32 {
    fn scale_factor(&self, font: &Face) -> f32 {
        let px_per_em = self.font_size * (96.0 / 72.0);
        px_per_em * font.height() as f32 / font.units_per_em() as f32
        //(self.font_size / 72.0 * 96.0) / units_per_em
    }

    pub fn get_face(&self) -> Face {
        let face = rustybuzz::Face::from_slice(&self.font_data, 0).unwrap();
        return face;
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
    }

    pub fn set_buffer_size(&mut self, width: u32, height: u32) {
        self.width = width as f32;
        self.height = height as f32;
    }

    fn shape(&self, font: &Face, buf: UnicodeBuffer) -> GlyphBuffer {
        //let face = rustybuzz::Face::from_slice(&self.font_data, 0).unwrap();
        let glyph_buffer = rustybuzz::shape(font, &[], buf);
        return glyph_buffer;
    }

    //fn layout(&self, text: &GlyphBuffer) -> Vec<Glyph> {
    //    let font = ab_glyph::FontRef::try_from_slice(&self.font_data).unwrap();
    //    //let font = self.font.as_scaled(self.font_size);
    //    let mut glyphs = Vec::new();
    //    let position = point(0.0, 0.0);
    //    let v_advance = font.height() + font.line_gap();
    //    let mut caret = position + point(0.0, font.ascent());
    //    let mut last_glyph: Option<Glyph> = None;
    //    for g in text.glyph_infos() {
    //        let gid = GlyphId(g.glyph_id as u16);
    //        let mut glyph = gid.with_scale(self.font_size);
    //        //if c.is_control() {
    //        //    if c == '\n' {
    //        //        caret = point(position.x, caret.y + v_advance);
    //        //        last_glyph = None;
    //        //    }
    //        //    continue;
    //        //}
    //        //let mut glyph = font.scaled_glyph(c);
    //        if let Some(previous) = last_glyph.take() {
    //            caret.x += font.kern(previous.id, gid);
    //        }
    //        glyph.position = caret;
    //
    //        //last_glyph = Some(glyph.clone());
    //        caret.x += font.h_advance(gid);
    //
    //        if caret.x > position.x + self.width {
    //            caret = point(position.x, caret.y + v_advance);
    //            glyph.position = caret;
    //            last_glyph = None;
    //            caret.x += font.h_advance(gid);
    //        }
    //
    //        glyphs.push(glyph);
    //    }
    //
    //    return glyphs;
    //}
    fn layout(&self, font: &Face, buf: GlyphBuffer) -> Vec<Glyph> {
        let mut glyphs = Vec::new();
        let mut caret = point(0.0, 0.0);
        let scale_factor = self.scale_factor(font);
        let scaled_height = font.height() as f32 / scale_factor;
        let v_advance = scaled_height + font.line_gap() as f32;
        println!(
            "unscaled_height: {}, scaled_height: {}, scale_factor: {}",
            font.height(),
            scaled_height,
            scale_factor,
        );
        let infos = buf.glyph_infos();
        let positions = buf.glyph_positions();

        for i in 0..infos.len() {
            let glyph_id = GlyphId(infos[i].glyph_id as u16);
            let mut pos = caret;
            let posit = positions[i];
            let bbox = font.glyph_bounding_box(glyph_id).unwrap();
            let hadv = posit.x_advance;
            let scaled_advance = hadv as f32 / scale_factor;
            //println!("{:?}", posit);
            caret.x += scaled_advance;
            //    let name = font.glyph_name(glyph_id);
            if caret.x >= self.width {
                caret.x = 0.0;
                caret.y += v_advance;
                pos = caret;
                caret.x += scaled_advance;
            }
            let glyph = Glyph {
                id: glyph_id,
                pos,
                bbox,
            };
            //println!("{:?}", pos);
            glyphs.push(glyph);
        }

        return glyphs;
    }

    pub fn set_text(&mut self, text: &str) {
        //self.buf.push_str(text);
    }

    pub fn raster(&self, text: &str) -> Vec<Glyph> {
        let font = self.get_face();
        let mut buf = UnicodeBuffer::new();
        buf.push_str(text);
        let newbuf = self.shape(&font, buf);
        return self.layout(&font, newbuf);
    }

    pub fn draw2(&self, glyphs: &[Glyph], surface: &mut SoftBuffer) {
        let font = self.get_face();

        let mut builder = Builder {
            pos: point(0.0, 0.0),
            rasteriser: Rasterizer::new(self.width as usize, self.height as usize),
        };

        let scale_factor = self.scale_factor(&font);

        for g in glyphs {
            //let bounds = g.bbox;
            //let max_x = bounds.x_max as f32 / scale_factor;
            //let max_y = bounds.y_max as f32 / scale_factor;
            //let min_x = bounds.x_min as f32 / scale_factor;
            //let min_y = bounds.y_min as f32 / scale_factor;
            //println!("x: {}-{}, y: {}-{}", min_x, min_y, max_x, max_y);
            //builder.rasteriser.reset(max_x as usize, max_y as usize);
            println!("pos: {:?}", g.pos);
            if let Some(r) = font.outline_glyph(g.id, &mut builder) {
                //println!("rect: {:?}", scale(g.pos, scale_factor));
                //builder.rasteriser.for_each_pixel_2d(|x, y, v| {
                //    let x = x as usize + min_x as usize;
                //    let y = y as usize + min_y as usize;
                //    let g = 255 - (v * 0xff as f32) as u32;
                //    let c = g | g << 8 | g << 16;
                //    surface[x as usize + y as usize * self.width as usize] = c;
                //});
            }
        }
    }

    //pub fn draw(&self, glyphs: &[Glyph], surface: &mut SoftBuffer) {
    //    let font = FontVec::try_from_vec(self.font_data).unwrap();
    //    let scale = PxScale::from(45.0);
    //    let scaled_font = font.as_scaled(scale);
    //    for g in glyphs {
    //    if let Some(og) = scaled_font.outline_glyph(g) {
    //        let bounds = og.px_bounds();
    //        og.draw(|x, y, v| {
    //            let x = x as f32 + bounds.min.x;
    //            let y = y as f32 + bounds.min.y;
    //            // There's still a possibility that the glyph clips the boundaries of the bitmap
    //            if x >= 0.0 && (x as usize) < px_width && y >= 0.0 && (y as usize) < px_height {
    //                // save the coverage alpha
    //                pixel_data[x as usize + y as usize * px_width] += v;
    //            }
    //        });
    //    }
    //}
}

fn scale(r: Rect, factor: f32) -> Rect {
    Rect {
        x_min: (r.x_min as f32 / factor) as i16,
        y_min: (r.y_min as f32 / factor) as i16,
        x_max: (r.x_max as f32 / factor) as i16,
        y_max: (r.y_max as f32 / factor) as i16,
    }
}
