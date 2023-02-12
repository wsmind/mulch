use egui::*;
use egui_demo_lib::DemoWindows;

pub struct Editor {
    egui_demo: DemoWindows,
}

impl Editor {
    pub fn new() -> Self {
        let egui_demo = egui_demo_lib::DemoWindows::default();

        Self { egui_demo }
    }

    pub fn run(&mut self, ctx: &Context) {
        self.egui_demo.ui(ctx);
    }
}
