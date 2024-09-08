use std::num::NonZeroU32;
use std::ops::Add;
use std::rc::Rc;

use softbuffer::Surface;
use ttf_parser::{Face, GlyphId};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

type SoftBufferType<'a> = softbuffer::Buffer<'a, Rc<Window>, Rc<Window>>;
const FONT_PATH: &str = "testfiles/Vollkorn-Regular.otf";
//const TEXT: &str = "Hello\njgy\naåäö";
const TEXT: &str = "Born in 1935 in Sceaux in the Paris suburbs, Delon was expelled from several schools before leaving at 14 to work in a butcher’s shop.\nAfter a stint in the navy (during which he saw combat in France’s colonial war in Vietnam), he was dishonourably discharged in 1956 and drifted into acting. He was spotted by Hollywood producer David O Selznick at Cannes and signed to a contract, but decided to try his luck in French cinema and made his debut with a small role in Yves Allégret’s 1957 thriller Send a Woman When the Devil Fails.";
const PARA1: &str = "Born in 1935 in Sceaux in the Paris suburbs, Delon was expelled from several schools before leaving at 14 to work in a butcher’s shop.";
const PARA2: &str = "After a stint in the navy (during which he saw combat in France’s colonial war in Vietnam), he was dishonourably discharged in 1956 and drifted into acting.";
const PARA3: &str = "He was spotted by Hollywood producer David O Selznick at Cannes and signed to a contract, but decided to try his luck in French cinema and made his debut with a small role in Yves Allégret’s 1957 thriller Send a Woman When the Devil Fails.";

pub struct Glyph {
    gid: GlyphId,
    pos: Point,
    dim: Rect,
}

#[derive(Clone, Copy, Default)]
struct Point(f32, f32);

#[derive(Clone, Copy, Default)]
struct Rect {
    min: Point,
    max: Point,
}

impl From<Point> for ab_glyph_rasterizer::Point {
    fn from(value: Point) -> Self {
        ab_glyph_rasterizer::Point {
            x: value.0,
            y: value.1,
        }
    }
}
impl Add<Point> for Point {
    type Output = Self;
    fn add(self, rhs: Point) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

struct Builder {
    pos: Point,
    offset: Point,
    scale: f32,
    last_move: Option<Point>,
    rasteriser: ab_glyph_rasterizer::Rasterizer,
}

impl ttf_parser::OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.pos = Point(x * self.scale, y * self.scale) + self.offset;
        self.last_move = Some(self.pos);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p = Point(x * self.scale, y * self.scale) + self.offset;
        self.rasteriser.draw_line(self.pos.into(), p.into());
        self.pos = p;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let p1 = Point(x1 * self.scale, y1 * self.scale) + self.offset;
        let p2 = Point(x2 * self.scale, y2 * self.scale) + self.offset;
        self.rasteriser
            .draw_quad(self.pos.into(), p1.into(), p2.into());
        self.pos = p2;
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) {
        let p1 = Point(x1 * self.scale, y1 * self.scale) + self.offset;
        let p2 = Point(x2 * self.scale, y2 * self.scale) + self.offset;
        let p3 = Point(x3 * self.scale, y3 * self.scale) + self.offset;
        self.rasteriser
            .draw_cubic(self.pos.into(), p1.into(), p2.into(), p3.into());
        self.pos = p3;
    }

    fn close(&mut self) {
        if let Some(m) = self.last_move.take() {
            self.rasteriser.draw_line(self.pos.into(), m.into());
        }
    }
}
pub struct GlyphHandler {
    font_data: Vec<u8>,
    width: u32,
    height: u32,
    pub font_size: f32,
    scale_factor: f32,
    raw_text: Vec<String>,
    typeset_text: Vec<Glyph>,
}
impl GlyphHandler {
    pub fn new(font_path: &str) -> Self {
        let font_data = std::fs::read(font_path).unwrap();
        let face = Face::parse(&font_data, 0).unwrap();

        let font_size = 28.0;
        let scale_factor = scale_factor(font_size, &face);

        Self {
            font_data,
            width: 640,
            height: 480,
            font_size,
            scale_factor,
            raw_text: vec![],
            typeset_text: vec![],
        }
    }

    pub fn get_face(&self) -> Face {
        let face = Face::parse(&self.font_data, 0).unwrap();
        return face;
    }

    fn set_buffer_size(&mut self, width: u32, height: u32) {
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

    fn set_text(&mut self, text: &str) {
        self.raw_text.push(text.to_string());
    }

    fn clear_text(&mut self) {
        self.raw_text.truncate(0);
        self.typeset_text.truncate(0);
    }

    fn typeset(&mut self) {
        let face = Face::parse(&self.font_data, 0).unwrap();
        self.typeset_text.truncate(0);

        //let mut caret = self.caret;
        let mut caret = Point::default();
        let scaled_height = face.height() as f32 * self.scale_factor;
        for para in self.raw_text.iter() {
            for c in para.chars() {
                let gido = face.glyph_index(c);
                if gido.is_none() {
                    continue;
                }
                let gid = gido.unwrap();
                let hadv = face.glyph_hor_advance(gid).unwrap() as f32 * self.scale_factor;
                if caret.0 + hadv >= self.width as f32 {
                    caret = Point(0.0, caret.1 + scaled_height);
                }
                let pos = caret;
                caret.0 += hadv;

                let x_min = 0.0;
                let x_max = face.glyph_hor_side_bearing(gid).unwrap() as f32
                    + face.glyph_hor_advance(gid).unwrap() as f32;
                let y_min = face.descender() as f32;
                let y_max = face.ascender() as f32;
                let dim = Rect {
                    min: Point(x_min, y_min),
                    max: Point(x_max, y_max),
                };

                self.typeset_text.push(Glyph { gid, pos, dim })
            }
            caret = Point(0.0, caret.1 + (1.0 * scaled_height));
        }
    }

    pub fn raster(&self, surface: &mut SoftBufferType) {
        let font = self.get_face();
        let bbox = font.global_bounding_box();
        let mut builder = Builder {
            pos: Point(0.0, 0.0),
            last_move: None,
            offset: Point(0.0, (-font.descender() as f32) * self.scale_factor),
            scale: self.scale_factor,
            rasteriser: ab_glyph_rasterizer::Rasterizer::new(
                bbox.x_max as usize,
                bbox.y_max as usize,
            ),
        };
        for g in self.typeset_text.iter() {
            let min = g.dim.min;
            let max = g.dim.max;
            let w = max.0 * self.scale_factor;
            let h = (max.1 - min.1) * self.scale_factor;
            builder.rasteriser.reset(w as usize, h as usize);
            if let Some(_) = font.outline_glyph(g.gid, &mut builder) {
                builder.rasteriser.for_each_pixel_2d(|x, y, v| {
                    let y = h as u32 - y;
                    let x = x + g.pos.0 as u32;
                    let y = y + g.pos.1 as u32;
                    let g = ((1.0 - v) * 255.0) as u8;
                    let c = g as u32 | (g as u32) << 8 | (g as u32) << 16;
                    let idx = x as usize + y as usize * self.width as usize;
                    surface[idx] = c;
                });
            }
        }
    }
}

pub struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    glyphs: GlyphHandler,
}

impl App {
    pub fn new() -> Self {
        let glyphs = GlyphHandler::new(FONT_PATH);
        Self {
            window: None,
            surface: None,
            glyphs,
        }
    }
    pub fn init(&mut self, event_loop: &ActiveEventLoop) {
        let window = Rc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        let context = softbuffer::Context::new(window.clone()).unwrap();
        self.surface = softbuffer::Surface::new(&context, window.clone()).ok();
        self.window = Some(window);
        self.glyphs.clear_text();
        self.glyphs.set_text(PARA1);
        self.glyphs.set_text(PARA2);
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init(event_loop);
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: key,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match key.as_ref() {
                Key::Character("+") => {
                    if let Some(win) = self.window.as_ref() {
                        let fs = self.glyphs.font_size;
                        self.glyphs.set_font_size(fs + 4.0);
                        win.request_redraw();
                    }
                }
                Key::Character("-") => {
                    if let Some(win) = self.window.as_ref() {
                        let fs = self.glyphs.font_size;
                        self.glyphs.set_font_size(fs - 4.0);
                        win.request_redraw();
                    }
                }
                Key::Named(NamedKey::Escape) => {
                    event_loop.exit();
                }
                _ => (),
            },
            WindowEvent::Resized(new_size) => {
                if let Some(surface) = self.surface.as_mut() {
                    surface
                        .resize(
                            NonZeroU32::new(new_size.width).unwrap(),
                            NonZeroU32::new(new_size.height).unwrap(),
                        )
                        .unwrap();
                    self.glyphs.set_buffer_size(new_size.width, new_size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(window) = self.window.as_ref() {
                    let size = window.inner_size();
                    if let Some(surface) = self.surface.as_mut() {
                        let mut surface_buffer = surface.buffer_mut().unwrap();

                        for x in 0..size.width {
                            for y in 0..size.height {
                                surface_buffer[x as usize + y as usize * size.width as usize] =
                                    0x00ffffff;
                            }
                        }

                        self.glyphs.raster(&mut surface_buffer);

                        surface_buffer.present().unwrap();
                    }
                }
            }
            _ => (),
        }
    }
}

fn scale_factor(font_size: f32, font: &Face) -> f32 {
    let px_per_em = font_size * (96.0 / 72.0);
    let units_per_em = font.units_per_em() as f32;
    px_per_em / units_per_em
}
fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();
    let _ = event_loop.run_app(&mut app);
}
