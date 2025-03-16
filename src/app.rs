use std::num::NonZeroU32;
use std::path::Path;
use std::rc::Rc;

// use crate::book_handler::BookHandler;
use crate::config::Config;
use crate::font::fonts::FontIndexer;
use crate::font::{TypesetConfig, Typesetter};
use softbuffer::Surface;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

pub struct App {
    _font_index: FontIndexer,
    // _book_path: PathBuf,
    // graphics
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    // text rendering
    // typesetter: Rc<RefCell<Typesetter>>,
    config: Config,
    typeset_config: TypesetConfig,
    // book content
    // book: BookHandler,
    // cur_chapter: Option<String>,
    // cur_page: usize,
    // content: Content,

    //new
    // book_handler: BookHandler,
    // renderer: DrawHandler
}

impl App {
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

                // if let Some(page) = self.book.page() {
                //     let wid = self.typeset_config.page_width as usize;
                //     page.raster(&self.typeset_config.family, wid, |c, idx| {
                //         surface_buffer[idx] = surface_buffer[idx].min(c);
                //     })
                //     .unwrap();
                // }

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
            self.typeset_config.page_width = new_size.width;
            self.typeset_config.page_height = new_size.height;
            // self.book.resize(new_size.width, new_size.height);
        }
    }
}
impl Default for App {
    fn default() -> Self {
        let config = Config::load_config();
        let indexer = FontIndexer::new("testfiles/fonts");
        let family = indexer.get_family(&config.family).unwrap();
        let path = Path::new("testfiles/epubs/frankenstein.epub");

        let tsconf = TypesetConfig {
            family,
            point_size: config.font_size,
            page_width: 640,
            page_height: 480,
            horizontal_margin: config.horizontal_margin,
            vertical_margin: config.vertical_margin,
        };
        let typesetter = Typesetter::new(&tsconf).unwrap();
        // let book = BookHandler::new(&path, typesetter);

        Self {
            _font_index: indexer,
            window: None,
            surface: None,
            config,
            // typesetter: Rc::clone(&typesetter),
            typeset_config: tsconf,
            // book,
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
                    // self.book.prev_page().unwrap();
                    if let Some(win) = self.window.as_ref() {
                        win.request_redraw();
                    }
                }
                Key::Named(NamedKey::ArrowRight) => {
                    // self.book.next_page().unwrap();
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
