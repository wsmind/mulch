mod eraser;
mod paintbrush;

pub use eraser::*;
pub use paintbrush::*;

pub trait Tool {
    fn icon(&self) -> &'static str;

    fn tooltip(&self) -> &'static str;

    fn shortcut(&self) -> egui::KeyboardShortcut;
}
