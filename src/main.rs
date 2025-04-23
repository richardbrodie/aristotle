use std::num::NonZeroU32;
use std::rc::Rc;

use softbuffer::Surface;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use aristotle_font::{
    indexer::FontIndexer, indexer::Indexer, GlyphHandler, Point, RenderingConfig, TextObject,
};

pub type SoftBufferType<'a> = softbuffer::Buffer<'a, Rc<Window>, Rc<Window>>;

const SHORT: &str = "Hello world, hello";
const LONG: &str = "Born in 1935 in Sceaux in the Paris suburbs, Delon was expelled from several schools before leaving at 14 to work in a butcher’s shop. After a stint in the navy (during which he saw combat in France’s colonial war in Vietnam), he was dishonourably discharged in 1956 and drifted into acting. He was spotted by Hollywood producer David O Selznick at Cannes and signed to a contract, but decided to try his luck in French cinema and made his debut with a small role in Yves Allégret’s 1957 thriller Send a Woman When the Devil Fails.";

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();
    let _ = event_loop.run_app(&mut app);
}

pub struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    glyphs: GlyphHandler,
    font_index: FontIndexer,
    text: Vec<TextObject>,
}

impl App {
    pub fn new() -> Self {
        let indexer = FontIndexer::new("testfiles");
        let config = RenderingConfig {
            point_size: 24.0,
            width: 640,
            height: 480,
            font: None,
        };
        let glyphs = GlyphHandler::new(&config);
        Self {
            window: None,
            surface: None,
            glyphs,
            text: vec![],
            font_index: indexer,
        }
    }
    pub fn init(&mut self, event_loop: &ActiveEventLoop) {
        let window = Rc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        let fam = self.font_index.get_family("Vollkorn").unwrap();
        self.glyphs.font = Some(fam);

        let context = softbuffer::Context::new(window.clone()).unwrap();
        self.surface = softbuffer::Surface::new(&context, window.clone()).ok();
        self.window = Some(window);
        //self.glyphs.clear_text();
        //self.glyphs.set_text(LONG);
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
                        //let mut config = self.glyphs.config();
                        //config.point_size += 2.0;
                        //self.glyphs.update_config(&config);
                        //win.request_redraw();
                    }
                }
                Key::Character("-") => {
                    if let Some(win) = self.window.as_ref() {
                        //let mut config = self.glyphs.config();
                        //config.point_size -= 2.0;
                        //self.glyphs.update_config(&config);
                        //win.request_redraw();
                    }
                }
                Key::Named(NamedKey::Space) => {
                    if let Some(win) = self.window.as_ref() {
                        let to = TextObject {
                            start_pos: Point::default(),
                            raw_text: SHORT.to_owned(),
                            ..Default::default()
                        };
                        self.text.push(to);
                        win.request_redraw();
                    }
                    //if let Some(win) = self.window.as_ref() {
                    //    self.glyphs.set_text("hello");
                    //    win.request_redraw();
                    //}
                    //if let Some(win) = self.window.as_ref() {
                    //    let mut config = self.glyphs.config();
                    //    if self.font == 0 {
                    //        self.font = 1;
                    //    } else {
                    //        self.font = 0;
                    //    }
                    //    config.font_path = FONT_PATHS[self.font].to_owned();
                    //    self.glyphs.update_config(&config);
                    //    win.request_redraw();
                    //}
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

                        let caret = Point::default();
                        if self.text.len() > 0 {
                            let t = self.glyphs.typeset(&self.text[0], caret).unwrap();
                            // caret = t.caret;
                            let _ = self.glyphs.raster(&t.glyphs, |x, y, z| {
                                let c = z as u32 | (z as u32) << 8 | (z as u32) << 16;
                                let idx =
                                    x as usize + y as usize * self.glyphs.canvas_width as usize;
                                surface_buffer[idx] = surface_buffer[idx].min(c);
                            });
                        }

                        surface_buffer.present().unwrap();
                        //event_loop.exit();
                    }
                }
            }
            _ => (),
        }
    }
}
