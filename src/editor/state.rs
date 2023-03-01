use crate::editor::tools;

pub struct EditorState {
    pub tools: Vec<Box<dyn tools::Tool>>,
    pub selected_tool: usize,

    // workaround egui modifier state being unreliable
    pub keyboard_modifiers: winit::event::ModifiersState,
}

impl Default for EditorState {
    fn default() -> Self {
        let tools: Vec<Box<dyn tools::Tool>> =
            vec![Box::new(tools::Paintbrush {}), Box::new(tools::Eraser {})];

        Self {
            tools,
            selected_tool: 0,
            keyboard_modifiers: winit::event::ModifiersState::default(),
        }
    }
}
