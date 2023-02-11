use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub struct App {}

impl App {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Mulch 3D")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .with_visible(false)
            .build(&event_loop)
            .unwrap();

        //let start_time = std::time::Instant::now();

        window.set_maximized(true);
        window.set_visible(true);

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => {}
            },

            Event::RedrawRequested(_) => {
                //let time = start_time.elapsed().as_secs_f64();
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            _ => {}
        });
    }
}
