use std::path::Path;
use std::rc::Rc;

use app::App;
use epub::{Content, Indexable, Node};
use winit::event_loop::{ControlFlow, EventLoop};

use winit::window::Window;

mod app;
// mod book_handler;
mod config;
mod epub;
mod font;
// mod paginate;

pub type SoftBufferType<'a> = softbuffer::Buffer<'a, Rc<Window>, Rc<Window>>;

fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    // let mut app = App::default();
    // let _ = event_loop.run_app(&mut app);
    let path = Path::new("testfiles/epubs/frankenstein.epub");
    let mut book = epub::Book::new(&path).unwrap();

    // let first_content = book.first().unwrap();
    // let id = first_content.id().to_owned();

    // let next_content = book.content(next.id()).unwrap();
    // list(&next_content);
}

fn list(c: &Content<'_>) {
    for e in c.iter() {
        println!("{}", e);
    }
}
