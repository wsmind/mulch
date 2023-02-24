use crate::editor::tools::Tool;

pub struct Eraser {}

impl Tool for Eraser {
    fn icon(&self) -> &'static str {
        "\u{f12d}"
    }
}
