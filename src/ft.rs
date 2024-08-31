use ab_glyph::{point, Font, FontVec, Glyph, ScaleFont};
use unicode_normalization::UnicodeNormalization;

use crate::SoftBuffer;

pub struct GlyphHandler {
    font: FontVec,
    width: f32,
    height: f32,
    pub font_size: f32,
}

impl GlyphHandler {
    pub fn new() -> Self {
        //let font_path = "testfiles/DejaVuSansMono.ttf";
        //let font_path = "testfiles/Mido.otf";
        let font_path = "testfiles/Vollkorn-Regular.otf";
        //let font_path = "testfiles/Exo2-Light.otf";
        //let font_path = "testfiles/OpenSans-Italic.ttf";
        //let font_path = "testfiles/SourceSansVariable-Roman.ttf";
        let data = std::fs::read(&font_path).unwrap();
        let font = FontVec::try_from_vec(data).unwrap_or_else(|_| {
            panic!("error constructing a Font from data at {:?}", font_path);
        });

        Self {
            font,
            width: 0.0,
            height: 0.0,
            font_size: 24.0,
        }
    }

    pub fn set_font_size(&mut self, size: f32) {
        self.font_size = size;
    }

    pub fn set_buffer_size(&mut self, width: u32, height: u32) {
        self.width = width as f32;
        self.height = height as f32;
    }

    pub fn render(&self, text: &str) -> Vec<Glyph> {
        let font = self.font.as_scaled(self.font_size);
        let scale = self.font.pt_to_px_scale(self.font_size).unwrap();
        let v_advance = font.height() + font.line_gap();
        println!(
            "unscaled_height: {}, scaled_height: {}, scale_factor: {}",
            self.font.height_unscaled(),
            font.height(),
            scale.x
        );

        let mut glyphs = Vec::new();
        let position = point(0.0, 0.0);
        let mut caret = position + point(0.0, font.ascent());
        let mut last_glyph: Option<Glyph> = None;
        //for c in text.chars() {
        for c in text.nfc() {
            if c.is_control() {
                if c == '\n' {
                    caret = point(position.x, caret.y + v_advance);
                    last_glyph = None;
                }
                continue;
            }
            let mut glyph = font.scaled_glyph(c);
            if let Some(previous) = last_glyph.take() {
                caret.x += font.kern(previous.id, glyph.id);
            }
            glyph.position = caret;

            last_glyph = Some(glyph.clone());
            let hadv = font.h_advance(glyph.id);
            caret.x += hadv;
            //println!("horizontal_advance: {}", hadv);

            if !c.is_whitespace() && caret.x > position.x + self.width {
                caret = point(position.x, caret.y + v_advance);
                glyph.position = caret;
                last_glyph = None;
                caret.x += font.h_advance(glyph.id);
            }
            //println!("{}: {:?}", c, glyph.position);

            glyphs.push(glyph);
        }

        return glyphs;
    }
    pub fn draw(&self, glyphs: &[Glyph], surface: &mut SoftBuffer) {
        for g in glyphs {
            if let Some(og) = self.font.outline_glyph(g.to_owned()) {
                let bounds = og.px_bounds();
                println!("{:?}", g.position);
                //og.draw(|x, y, v| {
                //    let x = x as f32 + bounds.min.x;
                //    let y = y as f32 + bounds.min.y;
                //    let g = 255 - (v * 0xff as f32) as u32;
                //    let c = g | g << 8 | g << 16;
                //    surface[x as usize + y as usize * self.width as usize] = c;
                //});
            }
        }
    }
}
