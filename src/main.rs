
use std::rc::Rc;

use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

use winit::window::Window;

mod app;
mod book;
mod config;
mod paginate;
mod epub;
mod font;

pub type SoftBufferType<'a> = softbuffer::Buffer<'a, Rc<Window>, Rc<Window>>;

fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    let _ = event_loop.run_app(&mut app);
}

