use crate::editor::tools;

pub struct EditorState {
    pub tools: Vec<Box<dyn tools::Tool>>,
    pub selected_tool: usize,
}

impl Default for EditorState {
    fn default() -> Self {
        let tools: Vec<Box<dyn tools::Tool>> =
            vec![Box::new(tools::Paintbrush {}), Box::new(tools::Eraser {})];

        Self {
            tools,
            selected_tool: 0,
        }
    }
}
