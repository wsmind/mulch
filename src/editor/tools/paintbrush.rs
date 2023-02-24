use crate::editor::tools::Tool;

pub struct PaintBrush {}

impl Tool for PaintBrush {
    fn icon(&self) -> &'static str {
        "\u{f1fc}"
    }
}
