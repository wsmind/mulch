mod eraser;
mod paintbrush;

pub use eraser::*;
pub use paintbrush::*;

pub trait Tool {
    fn icon(&self) -> &'static str;
}
