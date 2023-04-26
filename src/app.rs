use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::document;
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
        let mut doc = document::Document::default();

        doc.layers[0].voxel_grid.paint_cube((2, 3, 0), (9, 5, 2));
        doc.layers[0].voxel_grid.paint_cube((4, 1, 0), (7, 8, 1));
        doc.layers[0].voxel_grid.paint_cube((5, 4, 2), (6, 5, 5));
        doc.layers[0]
            .voxel_grid
            .paint_cube((30, 30, 1), (40, 40, 30));
        doc.layers[0].voxel_grid.paint_sphere((6, 6, 6), 2.5);

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
                let keyboard_modifiers = ui_context.modifiers_state;

                let ui_render_data = ui_context.run(&window, time, |ctx| {
                    editor.run(ctx, &mut doc, keyboard_modifiers)
                });

                renderer.render(&doc, &ui_render_data);
            }

            Event::MainEventsCleared => {
                window.request_redraw();
            }

            _ => {}
        });
    }
}
