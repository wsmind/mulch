use crate::editor::tools::Tool;

pub struct Paintbrush {}

impl Tool for Paintbrush {
    fn icon(&self) -> &'static str {
        "\u{f1fc}"
    }

    fn tooltip(&self) -> &'static str {
        "Paintbrush"
    }

    fn shortcut(&self) -> egui::KeyboardShortcut {
        egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::B)
    }
}
