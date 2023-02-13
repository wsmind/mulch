use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::editor;
use crate::render;
use crate::ui;

pub struct App {}

impl App {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Mulch 3D")
            .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
            .build(&event_loop)
            .unwrap();

        let window_size = window.inner_size();
        let mut renderer = render::Renderer::new(&window, [window_size.width, window_size.height]);
        let mut viewport = render::Viewport::default();

        let mut ui_context = ui::UiContext::new();
        let mut editor = editor::Editor::new();

        window.set_maximized(true);

        let start_time = std::time::Instant::now();

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => {
                    renderer.resize([new_size.width, new_size.height]);
                }
                _ => ui_context.handle_event(event),
            },

            Event::RedrawRequested(_) => {
                let time = start_time.elapsed().as_secs_f64();

                let ui_render_data =
                    ui_context.run(&window, time, |ctx| editor.run(ctx, &mut viewport));

                renderer.render(&viewport, &ui_render_data);
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            _ => {}
        });
    }
}
