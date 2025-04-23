use ab_glyph_rasterizer::{point, Point, Rasterizer};
use rustybuzz::ttf_parser::{GlyphId, Rect};
use rustybuzz::Face;
use rustybuzz::{GlyphBuffer, UnicodeBuffer};

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

    fn close(&mut self) {}
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
            font_data,
            width: 0.0,
            height: 0.0,
            font_size: 172.0,
        }
    }

    fn scale_factor(&self, font: &Face) -> f32 {
        let px_per_em = self.font_size * (96.0 / 72.0);
        let units_per_em = font.units_per_em() as f32;
        px_per_em / units_per_em
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
        let glyph_buffer = rustybuzz::shape(font, &[], buf);
        return glyph_buffer;
    }

    fn layout(&self, font: &Face, buf: GlyphBuffer) -> Vec<Glyph> {
        let mut glyphs = Vec::new();
        let scale_factor = self.scale_factor(font);
        //let ascent = font.height() as f32 * scale_factor;
        let mut caret = point(0.0, 0.0);
        let scaled_height = font.height() as f32 * scale_factor;
        let v_advance = scaled_height + font.line_gap() as f32;
        let infos = buf.glyph_infos();
        let positions = buf.glyph_positions();

        for i in 0..infos.len() {
            let glyph_id = GlyphId(infos[i].glyph_id as u16);
            let mut pos = caret;
            let posit = positions[i];
            let hadv = posit.x_advance;
            let scaled_advance = hadv as f32 * scale_factor;
            caret.x += scaled_advance;
            //let name = font.glyph_name(glyph_id);
            if caret.x >= self.width {
                caret.x = 0.0;
                caret.y += v_advance;
                pos = caret;
                caret.x += scaled_advance;
            }
            let bbox = font.glyph_bounding_box(glyph_id).unwrap();
            let glyph = Glyph {
                id: glyph_id,
                pos,
                bbox,
            };
            glyphs.push(glyph);
        }

        return glyphs;
    }

    pub fn raster(&self, text: &str) -> Vec<Glyph> {
        let font = self.get_face();
        let mut buf = UnicodeBuffer::new();
        buf.push_str(text);
        let newbuf = self.shape(&font, buf);
        return self.layout(&font, newbuf);
    }

    pub fn draw(&self, glyphs: &[Glyph], surface: &mut SoftBuffer) {
        let font = self.get_face();
        let scale_factor = self.scale_factor(&font);
        let mut builder = Builder {
            pos: point(0.0, 0.0),
            rasteriser: Rasterizer::new(0, 0),
        };

        for g in glyphs {
            let bounds = g.bbox;
            dbg!(bounds);
            let bx = bounds.x_max + bounds.x_min;
            let by = bounds.y_max;
            builder.rasteriser.reset(bx as usize, by as usize);
            if let Some(_) = font.outline_glyph(g.id, &mut builder) {
                builder.rasteriser.for_each_pixel_2d(|x, y, v| {
                    let y = by as u32 - y;
                    let x = (x as f32 * scale_factor) + g.pos.x;
                    let y = (y as f32 * scale_factor) + g.pos.y;
                    let g = 255 - (v.clamp(0.0, 1.0) * 255.0) as u32;
                    let n = g | (g << 8) | (g << 16);
                    let idx = x as usize + y as usize * self.width as usize;
                    surface[idx] = n;
                });
            }
        }
    }
}

fn scalep(r: Point, factor: f32) -> Point {
    Point {
        x: r.x * factor,
        y: r.y * factor,
    }
}
fn scaler(r: Rect, factor: f32) -> Rect {
    Rect {
        x_min: (r.x_min as f32 * factor) as i16,
        y_min: (r.y_min as f32 * factor) as i16,
        x_max: (r.x_max as f32 * factor) as i16,
        y_max: (r.y_max as f32 * factor) as i16,
    }
}
