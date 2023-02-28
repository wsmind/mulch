use egui::*;

pub struct ToolbarButton {
    selected: bool,
    icon: WidgetText,
}

impl ToolbarButton {
    pub fn new(selected: bool, icon: impl Into<WidgetText>) -> Self {
        Self {
            selected,
            icon: icon.into(),
        }
    }
}

impl Widget for ToolbarButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { selected, icon } = self;

        let text = icon.into_galley(ui, None, f32::INFINITY, TextStyle::Button);

        let mut desired_size = vec2(32.0, 32.0);
        desired_size.y = desired_size.y.at_least(ui.spacing().interact_size.y);
        let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());

        if ui.is_rect_visible(response.rect) {
            let text_pos = Layout::centered_and_justified(Direction::TopDown)
                .align_size_within_rect(text.size(), rect)
                .min;

            let visuals = ui.style().interact_selectable(&response, selected);

            if selected || response.hovered() || response.highlighted() || response.has_focus() {
                let rect = rect.expand(visuals.expansion);

                ui.painter().rect(
                    rect,
                    visuals.rounding,
                    visuals.weak_bg_fill,
                    visuals.bg_stroke,
                );
            }

            text.paint_with_visuals(ui.painter(), text_pos, &visuals);
        }

        response
    }
}
