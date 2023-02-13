use egui::*;

use crate::render;

pub struct Editor {
    //egui_demo: DemoWindows,
}

impl Editor {
    pub fn new() -> Self {
        // let egui_demo = egui_demo_lib::DemoWindows::default();

        // Self { egui_demo }
        Self {}
    }

    pub fn run(&mut self, ctx: &Context, viewport: &mut render::Viewport) {
        //self.egui_demo.ui(ctx);

        SidePanel::right("side_panel")
            .default_width(200.0)
            .show(&ctx, |ui| {
                ui.label("hello");
                ui.separator();
                ui.label("panel!");
            });

        CentralPanel::default()
            .frame(Frame::none())
            .show(&ctx, |ui| {
                viewport.rect = ui.max_rect();
                ui.label("viewport");
                ui.checkbox(&mut viewport.option, "enabled");
            });
    }
}
