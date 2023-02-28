use crate::editor::tools::Tool;

pub struct Eraser {}

impl Tool for Eraser {
    fn icon(&self) -> &'static str {
        "\u{f12d}"
    }

    fn tooltip(&self) -> &'static str {
        "Eraser"
    }

    fn shortcut(&self) -> egui::KeyboardShortcut {
        egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::E)
    }
}
