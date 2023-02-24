use egui::*;

use crate::document;

mod tools;

pub struct Editor {
    egui_demo: egui_demo_lib::DemoWindows,
    selected_layer: usize,
    layer_rename: bool,
    layer_name: String,

    tools: Vec<Box<dyn tools::Tool>>,
    selected_tool: usize,
}

impl Editor {
    pub fn new() -> Self {
        let egui_demo = egui_demo_lib::DemoWindows::default();

        let tools: Vec<Box<dyn tools::Tool>> =
            vec![Box::new(tools::PaintBrush {}), Box::new(tools::Eraser {})];

        Self {
            egui_demo,
            selected_layer: 0,
            layer_rename: false,
            layer_name: String::new(),
            tools,
            selected_tool: 0,
        }
    }

    pub fn run(&mut self, ctx: &Context, doc: &mut document::Document) {
        //self.egui_demo.ui(ctx);

        SidePanel::left("toolbar")
            .exact_width(32.0)
            .resizable(false)
            .show(ctx, |ui| {
                let style = ui.style_mut();
                style.text_styles.get_mut(&TextStyle::Button).unwrap().size = 20.0;
                style.spacing.item_spacing = Vec2::new(8.0, 8.0);
                for (i, tool) in self.tools.iter().enumerate() {
                    let selected = self.selected_tool == i;
                    if ui.selectable_label(selected, tool.icon()).clicked() {
                        self.selected_tool = i;
                    }
                }
            });

        SidePanel::right("side_panel")
            .default_width(200.0)
            .show(ctx, |ui| {
                ctx.style_ui(ui);
                ui.strong("\u{f5fd} Layers");
                ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for (i, layer) in doc.layers.iter_mut().enumerate() {
                            let is_selected = self.selected_layer == i;

                            let available_width = ui.available_width();
                            let response = ui.allocate_response(
                                vec2(available_width, 32.0),
                                Sense {
                                    click: true,
                                    drag: false,
                                    focusable: true,
                                },
                            );

                            let visuals = ui.style().interact_selectable(&response, is_selected);

                            // let text = WidgetText::from(layer.name.as_str()).into_galley(
                            //     ui,
                            //     Some(false),
                            //     0.0,
                            //     TextStyle::Body,
                            // );

                            if ui.is_rect_visible(response.rect) {
                                ui.painter().rect(
                                    response.rect,
                                    2.0,
                                    visuals.bg_fill,
                                    Stroke::NONE,
                                );

                                // let text_pos = ui
                                //     .layout()
                                //     .align_size_within_rect(
                                //         text.size(),
                                //         response.rect.shrink2(inner_rect),
                                //     )
                                //     .min;

                                // text.paint_with_visuals(ui.painter(), text_pos, &visuals);
                                //let inner_rect = response.rect.shrink2(vec2(8.0, 8.0));
                                let mut child_ui = ui.child_ui(response.rect, *ui.layout());

                                if is_selected && self.layer_rename == true {
                                    let edit = child_ui.add_sized(
                                        vec2(response.rect.width(), response.rect.height()),
                                        TextEdit::singleline(&mut self.layer_name)
                                            .margin(vec2(0.0, 0.0)),
                                    );
                                    if edit.lost_focus() {
                                        layer.name = self.layer_name.take();
                                        self.layer_rename = false;
                                    }
                                } else {
                                    child_ui.label(layer.name.as_str());
                                }
                            }

                            // let mut frame = Frame::none()
                            //     .fill(Color32::from_rgb(40, 40, 40))
                            //     .rounding(2.0);
                            // if is_selected {
                            //     frame.fill = Color32::from_rgb(60, 60, 60);
                            // }

                            // let response = frame
                            //     .show(ui, |ui| {
                            //         let label = ui.label(&layer.name);
                            //         ui.separator();
                            //     })
                            //     .response;

                            if response.clicked() {
                                self.selected_layer = i;
                                self.layer_rename = false;
                            }
                            if response.double_clicked() {
                                self.layer_name = layer.name.clone();
                                self.layer_rename = true;
                            }
                        }
                    });
                ui.separator();
            });

        CentralPanel::default()
            .frame(Frame::none())
            .show(&ctx, |ui| {
                doc.viewport.rect = ui.max_rect();
                ui.label("viewport");
                ui.checkbox(&mut doc.viewport.option, "enabled");
            });
    }
}
