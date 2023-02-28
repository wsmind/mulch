use egui::*;

use super::{state::EditorState, widgets::ToolbarButton};

pub struct Toolbar {}

impl Toolbar {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ctx: &Context, state: &mut EditorState) {
        let window = Window::new("Toolbar")
            .title_bar(false)
            .resizable(false)
            .anchor(Align2::LEFT_TOP, vec2(4.0, 4.0));
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

                let button = ToolbarButton::new(selected, tool.icon());
                if ui.add(button).on_hover_text(tooltip).clicked() {
                    state.selected_tool = i;
                }

                ctx.input_mut(|input| {
                    if input.consume_shortcut(&tool.shortcut()) {
                        state.selected_tool = i;
                    }
                })

                // if ui.selectable_label(selected, tool.icon()).clicked() {
                //     state.selected_tool = i;
                // }
            }
        });

        // SidePanel::left("toolbar")
        //     //.exact_width(32.0)
        //     .width_range(0.0..=200.0)
        //     .resizable(true)
        //     .show(ctx, |ui| {});
    }
}
