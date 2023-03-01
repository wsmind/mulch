use egui::*;

use super::state::EditorState;

pub struct Toolbar {}

impl Toolbar {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ctx: &Context, state: &mut EditorState) {
        let window_margin = ctx.style().spacing.window_margin.left;

        let window = Window::new("Toolbar")
            .title_bar(false)
            .resizable(false)
            .anchor(Align2::LEFT_TOP, vec2(window_margin, window_margin));
        window.show(ctx, |ui| {
            let style = ui.style_mut();
            style.text_styles.get_mut(&TextStyle::Button).unwrap().size = 22.0;
            style.spacing.item_spacing = vec2(0.0, 8.0);

            for (i, tool) in state.tools.iter().enumerate() {
                let selected = state.selected_tool == i;
                let tooltip = format!(
                    "{} ({})",
                    tool.tooltip(),
                    ctx.format_shortcut(&tool.shortcut())
                );

                let response = ui
                    .add_sized(
                        vec2(32.0, 32.0),
                        SelectableLabel::new(selected, tool.icon()),
                    )
                    .on_hover_text(tooltip);

                if response.clicked() {
                    state.selected_tool = i;
                }

                ctx.input_mut(|input| {
                    if input.consume_shortcut(&tool.shortcut()) {
                        state.selected_tool = i;
                    }
                });
            }
        });
    }
}
