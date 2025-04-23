use std::num::NonZeroU32;
use std::rc::Rc;

use softbuffer::Surface;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use aristotle_font::GlyphHandler;

pub type SoftBufferType<'a> = softbuffer::Buffer<'a, Rc<Window>, Rc<Window>>;

const FONT_PATHS: [&str; 4] = [
    "testfiles/Vollkorn-Regular.otf",
    "testfiles/Exo2-Light.otf",
    "testfiles/Mido.otf",
    "testfiles/OpenSans-Italic.ttf",
];
const SHORT: &str = "Hello world, hello";

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
}

impl App {
    pub fn new() -> Self {
        let glyphs = GlyphHandler::new(FONT_PATHS[0]);
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
        self.glyphs.set_text(SHORT);
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
                Key::Named(NamedKey::Space) => {
                    if let Some(win) = self.window.as_ref() {
                        self.glyphs.set_text("hello");
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

                        self.glyphs.raster(|x, y, z| {
                            let c = z as u32 | (z as u32) << 8 | (z as u32) << 16;
                            let idx = x as usize + y as usize * self.glyphs.width as usize;
                            surface_buffer[idx] = surface_buffer[idx].min(c);
                        });

                        surface_buffer.present().unwrap();
                    }
                }
            }
            _ => (),
        }
    }
}
