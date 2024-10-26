use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use aristotle_font::{ContentElement, Error, TypesetConfig, Typesetter};
use epub::{Book, Element};
use softbuffer::Surface;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use aristotle_font::fonts::{FontIndexer, Indexer};

use self::text::convert_content;

mod text;

pub type SoftBufferType<'a> = softbuffer::Buffer<'a, Rc<Window>, Rc<Window>>;

fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}

pub struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    _font_index: FontIndexer,
    text: Vec<ContentElement>,
    typesetter: Typesetter,
    _book_path: PathBuf,
    book: Book,
    cur_page: Option<Element>,
    typeset_config: TypesetConfig,
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
    fn typeset(&mut self) {
        let t = &mut self.typesetter;
        t.clear();
        for (i, to) in self.text.iter().enumerate() {
            match t.typeset(to) {
                Err(Error::ContentOverflow(idx)) => {
                    tracing::info!("element {} overflowed at char {}", i, idx);
                    break;
                }
                Err(Error::PageOverflow) => {
                    tracing::info!("filled page with element {}", i);
                    break;
                }
                Err(e) => tracing::error!("typeset error: {:?}", e),
                Ok(()) => (),
            }
        }
    }
}
impl Default for App {
    fn default() -> Self {
        let indexer = FontIndexer::new("testfiles/fonts");
        let family = indexer.get_family("Vollkorn").unwrap();
        let path = Path::new("testfiles/epubs/pride_and_prejudice.epub");
        let book = Book::new(path).unwrap();
        let cur_page = book.items().next();
        let c = TypesetConfig {
            point_size: 18.0,
            page_width: 640,
            page_height: 480,
            horizontal_margin: 16.0,
            vertical_margin: 16.0,
        };
        let typesetter = Typesetter::new(c, family).unwrap();
        Self {
            window: None,
            surface: None,
            text: vec![],
            typesetter,
            _font_index: indexer,
            _book_path: path.to_owned(),
            book,
            cur_page,
            typeset_config: c,
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
                Key::Named(NamedKey::Space) => {
                    if let Some(win) = self.window.as_ref() {
                        if let Some(cur) = &self.cur_page {
                            self.cur_page = self.book.next_item(cur.id());
                            tracing::info!("cur page: {:?}", self.cur_page);
                        }
                        let content = self
                            .book
                            .content(self.cur_page.as_ref().unwrap().id())
                            .unwrap();
                        self.text.clear();
                        for ce in content.content().iter() {
                            let c = convert_content(ce);
                            self.text.push(c);
                        }
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
                    self.typeset_config.page_width = new_size.width;
                    self.typeset_config.page_height = new_size.height;
                    self.typesetter
                        .set_buffer_size(new_size.width, new_size.height);
                    self.typeset();
                }
            }
            WindowEvent::RedrawRequested => {
                // TODO: don't run this every time
                self.typeset();
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

                        self.typesetter
                            .raster(|x, y, z| {
                                let c = z as u32 | (z as u32) << 8 | (z as u32) << 16;
                                let idx = x as usize
                                    + y as usize * self.typeset_config.page_width as usize;
                                surface_buffer[idx] = surface_buffer[idx].min(c);
                            })
                            .unwrap();

                        surface_buffer.present().unwrap();
                    }
                }
            }
            _ => (),
        }
    }
}
