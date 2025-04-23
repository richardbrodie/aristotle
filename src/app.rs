use std::num::NonZeroU32;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crate::book_handler::{self, BookHandler};
use crate::config::Config;
use crate::text::fonts::FontIndexer;
use crate::text::TypesetConfig;
use softbuffer::Surface;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

#[derive(Debug)]
pub enum Error {
    NoFont,
    BookHandler(book_handler::Error),
}

pub struct App {
    _font_index: FontIndexer,
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    config: Config,
    typeset_config: Arc<RwLock<TypesetConfig>>,
    book: BookHandler,
}

impl App {
    pub fn new(config: Config) -> Result<Self, Error> {
        let indexer = FontIndexer::new("testfiles/fonts");
        let family = indexer.get_family(&config.family).ok_or(Error::NoFont)?;
        let path = Path::new("testfiles/epubs/frankenstein.epub");

        let tsconf = TypesetConfig {
            family,
            point_size: config.font_size,
            page_width: 640,
            page_height: 480,
            horizontal_margin: config.horizontal_margin,
            vertical_margin: config.vertical_margin,
        };
        let tsconfig = Arc::new(RwLock::new(tsconf));
        let book = BookHandler::new(&path, tsconfig.clone()).map_err(|e| Error::BookHandler(e))?;

        Ok(Self {
            _font_index: indexer,
            window: None,
            surface: None,
            config,
            typeset_config: tsconfig,
            book,
        })
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
    fn redraw(&mut self) {
        if let Some(window) = self.window.as_ref() {
            let size = window.inner_size();
            if let Some(surface) = self.surface.as_mut() {
                let mut surface_buffer = surface.buffer_mut().unwrap();

                // fill every pixel with white
                for x in 0..size.width {
                    for y in 0..size.height {
                        surface_buffer[x as usize + y as usize * size.width as usize] = 0x00ffffff;
                    }
                }

                if let Some(page) = self.book.page() {
                    if let Ok(conf) = self.typeset_config.read() {
                        let wid = conf.page_width;
                        page.raster(&conf.family, wid, &mut surface_buffer).unwrap();
                    }
                }

                surface_buffer.present().unwrap();
            }
        }
    }
    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if let Some(surface) = self.surface.as_mut() {
            surface
                .resize(
                    NonZeroU32::new(new_size.width).unwrap(),
                    NonZeroU32::new(new_size.height).unwrap(),
                )
                .unwrap();
            if let Ok(mut conf) = self.typeset_config.write() {
                conf.page_width = new_size.width as usize;
                conf.page_height = new_size.height as usize;
            }
            self.book.repaginate().unwrap();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.init(event_loop);
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
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
                    if let Some(_win) = self.window.as_ref() {
                        //let mut config = self.glyphs.config();
                        //config.point_size += 2.0;
                        //self.glyphs.update_config(&config);
                        //win.request_redraw();
                    }
                }
                Key::Character("-") => {
                    if let Some(_win) = self.window.as_ref() {
                        //let mut config = self.glyphs.config();
                        //config.point_size -= 2.0;
                        //self.glyphs.update_config(&config);
                        //win.request_redraw();
                    }
                }
                //Key::Named(NamedKey::Space) => {
                //    tracing::info!("cur chapter: {:?}", self.cur_chapter);
                //    if let Some(cur) = &self.cur_chapter {
                //        self.cur_chapter = self.book.next_item(cur.id());
                //        tracing::info!("new chapter: {:?}", self.cur_chapter);
                //    }
                //    let content = self
                //        .book
                //        .content(self.cur_chapter.as_ref().unwrap().id())
                //        .unwrap();
                //    self.content.set_content(content);
                //    if let Some(win) = self.window.as_ref() {
                //        win.request_redraw();
                //    }
                //}
                Key::Named(NamedKey::ArrowLeft) => {
                    self.book.prev_page().unwrap();
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                }
                Key::Named(NamedKey::ArrowRight) => {
                    self.book.next_page().unwrap();
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                }
                Key::Named(NamedKey::Escape) => {
                    event_loop.exit();
                }
                _ => (),
            },
            WindowEvent::Resized(new_size) => {
                self.resize(new_size);
            }
            WindowEvent::RedrawRequested => {
                self.redraw();
            }
            _ => (),
        }
    }
}
