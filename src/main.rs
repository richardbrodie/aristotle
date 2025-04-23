mod app;
mod buzz;
//mod ct;
mod ft;

use std::rc::Rc;

use winit::event_loop::{ControlFlow, EventLoop};

use app::App;
use winit::window::Window;

type SoftBuffer<'a> = softbuffer::Buffer<'a, Rc<Window>, Rc<Window>>;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new();
    let _ = event_loop.run_app(&mut app);
}
