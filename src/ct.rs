use std::slice;

use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache};
use tiny_skia::{Paint, PixmapMut, Rect, Transform};

use crate::SoftBuffer;

pub struct FontHandler {
    font_system: FontSystem,
    swash_cache: SwashCache,
    metrics: Metrics,
    buffer_width: Option<f32>,
    buffer_height: Option<f32>,
}

impl FontHandler {
    pub fn new() -> Self {
        let mut font_system = FontSystem::new();
        let db = font_system.db_mut();
        db.load_fonts_dir("testfiles");

        let swash_cache = SwashCache::new();
        let metrics = Metrics::new(32.0, 32.0);
        let buffer_width = Some(640.0);
        let buffer_height = Some(32.0);
        Self {
            font_system,
            swash_cache,
            metrics,
            buffer_width,
            buffer_height,
        }
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        self.buffer_height = Some(height as f32);
        self.buffer_width = Some(width as f32);
    }

    pub fn render(&mut self, text: &str) -> Buffer {
        let mut buffer = Buffer::new(&mut self.font_system, self.metrics);
        buffer.set_size(&mut self.font_system, self.buffer_width, self.buffer_height);
        let attrs = Attrs::new().family(cosmic_text::Family::Serif);

        buffer.set_text(&mut self.font_system, &text, attrs, Shaping::Basic);
        buffer.shape_until_scroll(&mut self.font_system, true);

        return buffer;
    }

    pub fn draw(&mut self, buffer: &Buffer, surface: &mut SoftBuffer, width: u32, height: u32) {
        let surface_buffer_u8 = unsafe {
            slice::from_raw_parts_mut(surface.as_mut_ptr() as *mut u8, surface.len() * 4)
        };
        let mut pixmap = PixmapMut::from_bytes(surface_buffer_u8, width, height).unwrap();
        pixmap.fill(tiny_skia::Color::from_rgba8(0xff, 0xff, 0xff, 0xFF));

        let mut paint = Paint::default();
        let c = cosmic_text::Color::rgb(0, 0, 0);
        buffer.draw(
            &mut self.font_system,
            &mut self.swash_cache,
            c,
            |x, y, w, h, color| {
                paint.set_color_rgba8(color.b(), color.g(), color.r(), color.a());
                let r = Rect::from_xywh(
                    x as f32,
                    (y as f32) + (height as f32 / 2.0),
                    w as f32,
                    h as f32,
                );
                pixmap.fill_rect(r.unwrap(), &paint, Transform::identity(), None);
            },
        );
    }
}
