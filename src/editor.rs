mod panels;
mod state;
mod tools;

use egui::*;

use crate::document;

use self::panels::*;
use self::state::EditorState;

pub struct Editor {
    state: EditorState,
    toolbar: Toolbar,
    egui_demo: egui_demo_lib::DemoWindows,
    selected_layer: usize,
    layer_rename: bool,
    layer_name: String,
}

impl Editor {
    pub fn new() -> Self {
        let egui_demo = egui_demo_lib::DemoWindows::default();

        let state = EditorState::default();

        let toolbar = Toolbar::new();

        Self {
            state,
            toolbar,

            egui_demo,
            selected_layer: 0,
            layer_rename: false,
            layer_name: String::new(),
        }
    }

    pub fn run(
        &mut self,
        ctx: &Context,
        doc: &mut document::Document,
        keyboard_modifiers: winit::event::ModifiersState,
    ) {
        //self.egui_demo.ui(ctx);

        self.state.keyboard_modifiers = keyboard_modifiers;

        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
            });
        });

        self.toolbar.show(ctx, &mut self.state);

        SidePanel::right("side_panel")
            .default_width(200.0)
            .show(ctx, |ui| {
                //ctx.style_ui(ui);
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
                                    child_ui.add_sized(
                                        vec2(response.rect.width(), response.rect.height()),
                                        Label::new(layer.name.as_str()),
                                    );

                                    //child_ui.label(layer.name.as_str());
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

        let response = CentralPanel::default()
            .frame(Frame::none())
            .show(&ctx, |ui| {
                doc.viewport.rect = ui.max_rect();

                let response =
                    ui.allocate_response(doc.viewport.rect.size(), Sense::click_and_drag());

                // let mut child_ui = ui.child_ui(response.rect, *ui.layout());

                // child_ui.label("viewport");
                // child_ui.checkbox(&mut doc.viewport.grid_enabled, "grid enabled");

                response
            });

        let camera = &mut doc.viewport.camera;

        if response.inner.hovered() {
            ctx.input(|input| {
                let pointer_delta = input.pointer.delta() * ctx.pixels_per_point() * 0.02;
                let scroll_delta = input.scroll_delta.y * ctx.pixels_per_point() * 0.02;

                if input.pointer.middle_down() {
                    if keyboard_modifiers.shift() {
                        // pan
                        camera.translate_local_frame(glam::vec3(
                            -pointer_delta.x * 0.5,
                            pointer_delta.y * 0.5,
                            0.0,
                        ));
                    } else {
                        // orbit
                        camera.orbit(-pointer_delta.y * 0.2, -pointer_delta.x * 0.2, 8.0);
                    }
                }

                if scroll_delta != 0.0 {
                    // zoom
                    camera.translate_local_frame(glam::vec3(0.0, 0.0, -scroll_delta));
                }
            })
        }

        let window_margin = ctx.style().spacing.window_margin.left;
        Window::new("Viewport Settings")
            .anchor(Align2::RIGHT_TOP, vec2(-window_margin, window_margin))
            .default_width(200.0)
            .vscroll(true)
            .default_open(false)
            .show(ctx, |ui| {
                ui.strong("Camera");
                egui::Grid::new("transform_grid")
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Position");
                        ui.add(DragValue::new(&mut camera.position.x).speed(0.1));
                        ui.add(DragValue::new(&mut camera.position.y).speed(0.1));
                        ui.add(DragValue::new(&mut camera.position.z).speed(0.1));
                        ui.end_row();

                        ui.label("Pitch");
                        let previous_pitch = camera.pitch.to_degrees();
                        let mut pitch = previous_pitch;
                        ui.add(
                            DragValue::new(&mut pitch)
                                .speed(1.0)
                                .suffix("°")
                                .clamp_range(-89..=89),
                        );
                        if pitch != previous_pitch {
                            camera.pitch = pitch.to_radians()
                        }
                        ui.end_row();

                        ui.label("Yaw");
                        ui.drag_angle(&mut camera.yaw);
                        ui.end_row();
                    });
                ui.separator();
                ui.strong("Projection");
                egui::Grid::new("projection_grid")
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Focal Angle");

                        let previous_fovy = camera.fovy.to_degrees();
                        let mut fovy = previous_fovy;
                        ui.add(
                            DragValue::new(&mut fovy)
                                .speed(1.0)
                                .suffix("°")
                                .clamp_range(1.0..=179.0),
                        );
                        if fovy != previous_fovy {
                            camera.fovy = fovy.to_radians()
                        }
                        ui.end_row();

                        ui.label("Clip range");
                        ui.add(
                            DragValue::new(&mut camera.near)
                                .speed(0.1)
                                .clamp_range(0.01..=camera.far - 0.01),
                        );
                        ui.add(
                            DragValue::new(&mut camera.far)
                                .speed(0.1)
                                .clamp_range(camera.near + 0.01..=1000000.0),
                        );
                        ui.end_row();
                    });
                ui.separator();
                ui.strong("Display");
                ui.checkbox(&mut doc.viewport.grid_enabled, "Grid");
            });
    }
}
