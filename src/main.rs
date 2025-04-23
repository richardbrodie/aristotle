use std::rc::Rc;

use app::App;
use config::Config;
use winit::event_loop::{ControlFlow, EventLoop};

use winit::window::Window;

mod app;
mod book_handler;
mod config;
mod draw;
mod epub;
mod page;
mod text;

pub type SoftBufferType<'a> = softbuffer::Buffer<'a, Rc<Window>, Rc<Window>>;

fn main() {
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let config = Config::load_config().unwrap();
    let Ok(mut app) = App::new(config) else {
        tracing::error!("cannot create app");
        return;
    };
    if let Err(e) = event_loop.run_app(&mut app) {
        tracing::error!("app: {:?}", e);
    }
}
