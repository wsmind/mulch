use clipboard_win::{formats, get_clipboard, set_clipboard};
use egui::*;
use winit::{dpi::PhysicalSize, event::*};

pub struct UiRenderData {
    pub textures_delta: TexturesDelta,
    pub clipped_primitives: Vec<ClippedPrimitive>,
    pub pixels_per_point: f32,
}

pub struct UiContext {
    ctx: Context,
    events: Vec<egui::Event>,
    last_mouse_position: Pos2,
    initial_pixels_per_point: f32,
    modifiers_state: ModifiersState,
}

impl UiContext {
    pub fn new() -> Self {
        let ctx = Context::default();

        let initial_pixels_per_point = 1.10; // default to 10% zoom
        ctx.set_pixels_per_point(initial_pixels_per_point);

        Self {
            ctx,
            events: vec![],
            last_mouse_position: pos2(0.0, 0.0),
            initial_pixels_per_point,
            modifiers_state: ModifiersState::empty(),
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let Some(keycode) = input.virtual_keycode {
                    let pressed = input.state == ElementState::Pressed;

                    if let Some(key) = winit_to_egui_keycode(keycode) {
                        let event = egui::Event::Key {
                            key,
                            pressed: pressed,
                            repeat: false,
                            modifiers: winit_to_egui_modifiers(self.modifiers_state),
                        };
                        self.events.push(event);
                    }

                    if pressed {
                        if self.modifiers_state.ctrl() && keycode == VirtualKeyCode::C {
                            self.events.push(egui::Event::Copy);
                        } else if self.modifiers_state.ctrl() && keycode == VirtualKeyCode::X {
                            self.events.push(egui::Event::Cut);
                        } else if self.modifiers_state.ctrl() && keycode == VirtualKeyCode::V {
                            let clipboard_data = get_clipboard(formats::Unicode).unwrap();
                            self.events.push(egui::Event::Paste(clipboard_data));
                        }
                    }
                }
            }

            WindowEvent::ReceivedCharacter(character) => {
                if !character.is_control() {
                    self.events.push(egui::Event::Text(character.to_string()));
                }
            }

            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers_state = *modifiers;
            }

            WindowEvent::CursorMoved { position, .. } => {
                let pixels_per_point = self.ctx.pixels_per_point();
                let pos = egui::pos2(
                    position.x as f32 / pixels_per_point,
                    position.y as f32 / pixels_per_point,
                );
                self.last_mouse_position = pos;
                self.events.push(egui::Event::PointerMoved(pos));
            }

            WindowEvent::MouseInput { state, button, .. } => {
                self.events.push(egui::Event::PointerButton {
                    pos: self.last_mouse_position,
                    button: match button {
                        MouseButton::Left => egui::PointerButton::Primary,
                        MouseButton::Right => egui::PointerButton::Secondary,
                        MouseButton::Middle => egui::PointerButton::Middle,
                        _ => egui::PointerButton::Extra1,
                    },
                    pressed: match state {
                        ElementState::Pressed => true,
                        ElementState::Released => false,
                    },
                    modifiers: Default::default(),
                })
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let pixels_per_point = self.ctx.pixels_per_point();
                let point_delta = match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        (x * 40.0 / pixels_per_point, y * 40.0 / pixels_per_point)
                    }
                    MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition { x, y }) => {
                        (*x as f32 / pixels_per_point, *y as f32 / pixels_per_point)
                    }
                };
                self.events.push(egui::Event::Scroll(point_delta.into()))
            }

            _ => {}
        }
    }

    pub fn run(
        &mut self,
        window_size: PhysicalSize<u32>,
        time: f64,
        run_ui: impl FnOnce(&Context),
    ) -> UiRenderData {
        let events = self.events.clone();
        self.events.clear();

        let pixels_per_point = self.ctx.pixels_per_point();

        let egui_input = egui::RawInput {
            screen_rect: Some(egui::Rect {
                min: egui::pos2(0.0, 0.0),
                max: egui::pos2(
                    window_size.width as f32 / pixels_per_point,
                    window_size.height as f32 / pixels_per_point,
                ),
            }),
            pixels_per_point: None,
            time: Some(time),
            events,
            ..Default::default()
        };
        let egui_output = self.ctx.run(egui_input, |ctx| {
            egui::gui_zoom::zoom_with_keyboard_shortcuts(&ctx, Some(self.initial_pixels_per_point));
            run_ui(ctx);
        });

        let copied_text = &egui_output.platform_output.copied_text;
        if !copied_text.is_empty() {
            set_clipboard(formats::Unicode, copied_text).unwrap();
        }

        let textures_delta = egui_output.textures_delta;
        let clipped_primitives = self.ctx.tessellate(egui_output.shapes);

        UiRenderData {
            textures_delta,
            clipped_primitives,
            pixels_per_point,
        }
    }
}

fn winit_to_egui_keycode(key: VirtualKeyCode) -> Option<egui::Key> {
    Some(match key {
        VirtualKeyCode::Down => egui::Key::ArrowDown,
        VirtualKeyCode::Left => egui::Key::ArrowLeft,
        VirtualKeyCode::Right => egui::Key::ArrowRight,
        VirtualKeyCode::Up => egui::Key::ArrowUp,

        VirtualKeyCode::Escape => egui::Key::Escape,
        VirtualKeyCode::Tab => egui::Key::Tab,
        VirtualKeyCode::Back => egui::Key::Backspace,
        VirtualKeyCode::Return => egui::Key::Enter,
        VirtualKeyCode::Space => egui::Key::Space,

        VirtualKeyCode::Insert => egui::Key::Insert,
        VirtualKeyCode::Delete => egui::Key::Delete,
        VirtualKeyCode::Home => egui::Key::Home,
        VirtualKeyCode::End => egui::Key::End,
        VirtualKeyCode::PageUp => egui::Key::PageUp,
        VirtualKeyCode::PageDown => egui::Key::PageDown,

        // The virtual keycode for the Minus key.
        VirtualKeyCode::Minus | VirtualKeyCode::NumpadSubtract => egui::Key::Minus,
        // The virtual keycode for the Plus/Equals key.
        VirtualKeyCode::Equals | VirtualKeyCode::NumpadAdd => egui::Key::PlusEquals,

        // Either from the main row or from the numpad.
        VirtualKeyCode::Key0 | VirtualKeyCode::Numpad0 => egui::Key::Num0,
        VirtualKeyCode::Key1 | VirtualKeyCode::Numpad1 => egui::Key::Num1,
        VirtualKeyCode::Key2 | VirtualKeyCode::Numpad2 => egui::Key::Num2,
        VirtualKeyCode::Key3 | VirtualKeyCode::Numpad3 => egui::Key::Num3,
        VirtualKeyCode::Key4 | VirtualKeyCode::Numpad4 => egui::Key::Num4,
        VirtualKeyCode::Key5 | VirtualKeyCode::Numpad5 => egui::Key::Num5,
        VirtualKeyCode::Key6 | VirtualKeyCode::Numpad6 => egui::Key::Num6,
        VirtualKeyCode::Key7 | VirtualKeyCode::Numpad7 => egui::Key::Num7,
        VirtualKeyCode::Key8 | VirtualKeyCode::Numpad8 => egui::Key::Num8,
        VirtualKeyCode::Key9 | VirtualKeyCode::Numpad9 => egui::Key::Num9,

        VirtualKeyCode::A => egui::Key::A,
        VirtualKeyCode::B => egui::Key::B,
        VirtualKeyCode::C => egui::Key::C,
        VirtualKeyCode::D => egui::Key::D,
        VirtualKeyCode::E => egui::Key::E,
        VirtualKeyCode::F => egui::Key::F,
        VirtualKeyCode::G => egui::Key::G,
        VirtualKeyCode::H => egui::Key::H,
        VirtualKeyCode::I => egui::Key::I,
        VirtualKeyCode::J => egui::Key::J,
        VirtualKeyCode::K => egui::Key::K,
        VirtualKeyCode::L => egui::Key::L,
        VirtualKeyCode::M => egui::Key::M,
        VirtualKeyCode::N => egui::Key::N,
        VirtualKeyCode::O => egui::Key::O,
        VirtualKeyCode::P => egui::Key::P,
        VirtualKeyCode::Q => egui::Key::Q,
        VirtualKeyCode::R => egui::Key::R,
        VirtualKeyCode::S => egui::Key::S,
        VirtualKeyCode::T => egui::Key::T,
        VirtualKeyCode::U => egui::Key::U,
        VirtualKeyCode::V => egui::Key::V,
        VirtualKeyCode::W => egui::Key::W,
        VirtualKeyCode::X => egui::Key::X,
        VirtualKeyCode::Y => egui::Key::Y,
        VirtualKeyCode::Z => egui::Key::Z,

        VirtualKeyCode::F1 => egui::Key::F1,
        VirtualKeyCode::F2 => egui::Key::F2,
        VirtualKeyCode::F3 => egui::Key::F3,
        VirtualKeyCode::F4 => egui::Key::F4,
        VirtualKeyCode::F5 => egui::Key::F5,
        VirtualKeyCode::F6 => egui::Key::F6,
        VirtualKeyCode::F7 => egui::Key::F7,
        VirtualKeyCode::F8 => egui::Key::F8,
        VirtualKeyCode::F9 => egui::Key::F9,
        VirtualKeyCode::F10 => egui::Key::F10,
        VirtualKeyCode::F11 => egui::Key::F11,
        VirtualKeyCode::F12 => egui::Key::F12,
        VirtualKeyCode::F13 => egui::Key::F13,
        VirtualKeyCode::F14 => egui::Key::F14,
        VirtualKeyCode::F15 => egui::Key::F15,
        VirtualKeyCode::F16 => egui::Key::F16,
        VirtualKeyCode::F17 => egui::Key::F17,
        VirtualKeyCode::F18 => egui::Key::F18,
        VirtualKeyCode::F19 => egui::Key::F19,
        VirtualKeyCode::F20 => egui::Key::F20,

        _ => return None,
    })
}

fn winit_to_egui_modifiers(modifiers: ModifiersState) -> egui::Modifiers {
    egui::Modifiers {
        alt: modifiers.alt(),
        ctrl: modifiers.ctrl(),
        shift: modifiers.shift(),
        mac_cmd: false,
        command: modifiers.ctrl(),
    }
}
