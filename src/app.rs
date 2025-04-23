use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crate::book_handler::{self, BookHandler};
use crate::config::Config;
use crate::draw::Canvas;
use crate::text::fonts::FontIndexer;
use crate::text::geom::Rect;
use crate::text::TypesetConfig;
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalSize, Size};
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
    canvas: Option<Canvas>,
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
            page_height: 800,
            horizontal_margin: config.horizontal_margin,
            vertical_margin: config.vertical_margin,
        };
        let tsconfig = Arc::new(RwLock::new(tsconf));
        let book = BookHandler::new(&path, tsconfig.clone()).map_err(|e| Error::BookHandler(e))?;

        Ok(Self {
            _font_index: indexer,
            window: None,
            canvas: None,
            config,
            typeset_config: tsconfig,
            book,
        })
    }

    pub fn init(&mut self, event_loop: &ActiveEventLoop) {
        let size = PhysicalSize {
            width: self.config.page_width as u32,
            height: self.config.page_height as u32,
        };
        let attrs = Window::default_attributes().with_inner_size(Size::Physical(size));
        let window = Rc::new(event_loop.create_window(attrs).unwrap());

        self.canvas = Canvas::new(
            window.clone(),
            Rect {
                width: size.width as usize,
                height: size.height as usize,
            },
        )
        .ok();

        self.window = Some(window);
    }
    fn redraw(&mut self) {
        let Some(page) = self.book.page() else {
            tracing::info!("no book pages");
            return;
        };
        let Some(canvas) = self.canvas.as_mut() else {
            tracing::error!("canvas is None?");
            return;
        };
        let Ok(config) = self.typeset_config.read() else {
            tracing::warn!("couldn't lock typset_config");
            return;
        };
        canvas.blank().unwrap();
        page.raster(&config.family, canvas).unwrap();
        canvas.present().unwrap();
    }
    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        let Some(canvas) = &mut self.canvas else {
            return;
        };
        canvas.resize(new_size);
        if let Ok(mut conf) = self.typeset_config.write() {
            conf.page_width = new_size.width as usize;
            conf.page_height = new_size.height as usize;
        }
        self.book.repaginate().unwrap();
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
