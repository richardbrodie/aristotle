use std::num::NonZeroU32;
use std::rc::Rc;

use ab_glyph::{Font, FontVec};
use softbuffer::Surface;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use crate::buzz::GlyphHandler as BuzzHandler;
use crate::ft::GlyphHandler as FtHandler;

//const TEXT: &str = "abcdefABCDEF";
const TEXT: &str = "0Oo";
const LONGLONGTEXT: &str = "Born in 1935 in Sceaux in the Paris suburbs, Delon was expelled from several schools before leaving at 14 to work in a butcher’s shop. After a stint in the navy (during which he saw combat in France’s colonial war in Vietnam), he was dishonourably discharged in 1956 and drifted into acting. He was spotted by Hollywood producer David O Selznick at Cannes and signed to a contract, but decided to try his luck in French cinema and made his debut with a small role in Yves Allégret’s 1957 thriller Send a Woman When the Devil Fails.";

pub struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    ft: FtHandler,
    buzz: BuzzHandler,
}

impl App {
    pub fn new() -> Self {
        let ft = FtHandler::new();
        let buzz = BuzzHandler::new();
        Self {
            window: None,
            surface: None,
            ft,
            buzz,
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
                        let fs = self.buzz.font_size;
                        //self.ft.set_font_size(fs + 1.0);
                        self.buzz.set_font_size(fs + 4.0);
                        win.request_redraw();
                    }
                }
                Key::Character("-") => {
                    if let Some(win) = self.window.as_ref() {
                        let fs = self.buzz.font_size;
                        //self.ft.set_font_size(fs - 1.0);
                        self.buzz.set_font_size(fs - 4.0);
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
                    self.ft.set_buffer_size(new_size.width, new_size.height);
                    self.buzz.set_buffer_size(new_size.width, new_size.height);
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
                                    u32::MAX;
                            }
                        }

                        // ft
                        //let f = self.ft.render(TEXT);
                        //self.ft.draw(&f, &mut surface_buffer);

                        // buzz
                        let b = self.buzz.raster(TEXT);
                        self.buzz.draw(&b, &mut surface_buffer);

                        surface_buffer.present().unwrap();
                        //event_loop.exit();
                    }
                }
            }
            _ => (),
        }
    }
}
