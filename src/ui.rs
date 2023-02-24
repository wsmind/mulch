use clipboard_win::{formats, get_clipboard, set_clipboard};
use egui::*;
use winit::event::*;

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
    current_cursor_icon: Option<CursorIcon>,
}

impl UiContext {
    pub fn new() -> Self {
        let ctx = Context::default();

        let initial_pixels_per_point = 1.10; // default to 10% zoom
        ctx.set_pixels_per_point(initial_pixels_per_point);

        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "text_font".to_owned(),
            FontData::from_static(include_bytes!("../fonts/NotoSans-Light.ttf")),
        );

        fonts.font_data.insert(
            "icon_font".to_owned(),
            FontData::from_static(include_bytes!("../fonts/Font Awesome 6 Free-Solid-900.otf")),
        );

        let families = fonts.families.get_mut(&FontFamily::Proportional).unwrap();

        families.insert(0, "text_font".to_owned());
        families.insert(1, "icon_font".to_owned());

        ctx.set_fonts(fonts);

        Self {
            ctx,
            events: vec![],
            last_mouse_position: pos2(0.0, 0.0),
            initial_pixels_per_point,
            modifiers_state: ModifiersState::empty(),
            current_cursor_icon: None,
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
                let pos = pos2(
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
                        MouseButton::Left => PointerButton::Primary,
                        MouseButton::Right => PointerButton::Secondary,
                        MouseButton::Middle => PointerButton::Middle,
                        _ => PointerButton::Extra1,
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
        window: &winit::window::Window,
        time: f64,
        run_ui: impl FnOnce(&Context),
    ) -> UiRenderData {
        let events = self.events.clone();
        self.events.clear();

        let window_size = window.inner_size();
        let pixels_per_point = self.ctx.pixels_per_point();

        let input = RawInput {
            screen_rect: Some(Rect {
                min: pos2(0.0, 0.0),
                max: pos2(
                    window_size.width as f32 / pixels_per_point,
                    window_size.height as f32 / pixels_per_point,
                ),
            }),
            pixels_per_point: None,
            time: Some(time),
            events,
            ..Default::default()
        };
        let output = self.ctx.run(input, |ctx| {
            gui_zoom::zoom_with_keyboard_shortcuts(&ctx, Some(self.initial_pixels_per_point));
            run_ui(ctx);
        });

        let copied_text = &output.platform_output.copied_text;
        if !copied_text.is_empty() {
            set_clipboard(formats::Unicode, copied_text).unwrap();
        }

        if self.current_cursor_icon != Some(output.platform_output.cursor_icon) {
            self.current_cursor_icon = Some(output.platform_output.cursor_icon);
            match egui_to_winit_cursor_icon(output.platform_output.cursor_icon) {
                None => window.set_cursor_visible(false),
                Some(cursor_icon) => {
                    window.set_cursor_icon(cursor_icon);
                    window.set_cursor_visible(true);
                }
            }
        }

        let textures_delta = output.textures_delta;
        let clipped_primitives = self.ctx.tessellate(output.shapes);

        UiRenderData {
            textures_delta,
            clipped_primitives,
            pixels_per_point,
        }
    }
}

fn winit_to_egui_keycode(key: VirtualKeyCode) -> Option<Key> {
    Some(match key {
        VirtualKeyCode::Down => Key::ArrowDown,
        VirtualKeyCode::Left => Key::ArrowLeft,
        VirtualKeyCode::Right => Key::ArrowRight,
        VirtualKeyCode::Up => Key::ArrowUp,

        VirtualKeyCode::Escape => Key::Escape,
        VirtualKeyCode::Tab => Key::Tab,
        VirtualKeyCode::Back => Key::Backspace,
        VirtualKeyCode::Return => Key::Enter,
        VirtualKeyCode::Space => Key::Space,

        VirtualKeyCode::Insert => Key::Insert,
        VirtualKeyCode::Delete => Key::Delete,
        VirtualKeyCode::Home => Key::Home,
        VirtualKeyCode::End => Key::End,
        VirtualKeyCode::PageUp => Key::PageUp,
        VirtualKeyCode::PageDown => Key::PageDown,

        // The virtual keycode for the Minus key.
        VirtualKeyCode::Minus | VirtualKeyCode::NumpadSubtract => Key::Minus,
        // The virtual keycode for the Plus/Equals key.
        VirtualKeyCode::Equals | VirtualKeyCode::NumpadAdd => Key::PlusEquals,

        // Either from the main row or from the numpad.
        VirtualKeyCode::Key0 | VirtualKeyCode::Numpad0 => Key::Num0,
        VirtualKeyCode::Key1 | VirtualKeyCode::Numpad1 => Key::Num1,
        VirtualKeyCode::Key2 | VirtualKeyCode::Numpad2 => Key::Num2,
        VirtualKeyCode::Key3 | VirtualKeyCode::Numpad3 => Key::Num3,
        VirtualKeyCode::Key4 | VirtualKeyCode::Numpad4 => Key::Num4,
        VirtualKeyCode::Key5 | VirtualKeyCode::Numpad5 => Key::Num5,
        VirtualKeyCode::Key6 | VirtualKeyCode::Numpad6 => Key::Num6,
        VirtualKeyCode::Key7 | VirtualKeyCode::Numpad7 => Key::Num7,
        VirtualKeyCode::Key8 | VirtualKeyCode::Numpad8 => Key::Num8,
        VirtualKeyCode::Key9 | VirtualKeyCode::Numpad9 => Key::Num9,

        VirtualKeyCode::A => Key::A,
        VirtualKeyCode::B => Key::B,
        VirtualKeyCode::C => Key::C,
        VirtualKeyCode::D => Key::D,
        VirtualKeyCode::E => Key::E,
        VirtualKeyCode::F => Key::F,
        VirtualKeyCode::G => Key::G,
        VirtualKeyCode::H => Key::H,
        VirtualKeyCode::I => Key::I,
        VirtualKeyCode::J => Key::J,
        VirtualKeyCode::K => Key::K,
        VirtualKeyCode::L => Key::L,
        VirtualKeyCode::M => Key::M,
        VirtualKeyCode::N => Key::N,
        VirtualKeyCode::O => Key::O,
        VirtualKeyCode::P => Key::P,
        VirtualKeyCode::Q => Key::Q,
        VirtualKeyCode::R => Key::R,
        VirtualKeyCode::S => Key::S,
        VirtualKeyCode::T => Key::T,
        VirtualKeyCode::U => Key::U,
        VirtualKeyCode::V => Key::V,
        VirtualKeyCode::W => Key::W,
        VirtualKeyCode::X => Key::X,
        VirtualKeyCode::Y => Key::Y,
        VirtualKeyCode::Z => Key::Z,

        VirtualKeyCode::F1 => Key::F1,
        VirtualKeyCode::F2 => Key::F2,
        VirtualKeyCode::F3 => Key::F3,
        VirtualKeyCode::F4 => Key::F4,
        VirtualKeyCode::F5 => Key::F5,
        VirtualKeyCode::F6 => Key::F6,
        VirtualKeyCode::F7 => Key::F7,
        VirtualKeyCode::F8 => Key::F8,
        VirtualKeyCode::F9 => Key::F9,
        VirtualKeyCode::F10 => Key::F10,
        VirtualKeyCode::F11 => Key::F11,
        VirtualKeyCode::F12 => Key::F12,
        VirtualKeyCode::F13 => Key::F13,
        VirtualKeyCode::F14 => Key::F14,
        VirtualKeyCode::F15 => Key::F15,
        VirtualKeyCode::F16 => Key::F16,
        VirtualKeyCode::F17 => Key::F17,
        VirtualKeyCode::F18 => Key::F18,
        VirtualKeyCode::F19 => Key::F19,
        VirtualKeyCode::F20 => Key::F20,

        _ => return None,
    })
}

fn winit_to_egui_modifiers(modifiers: ModifiersState) -> Modifiers {
    Modifiers {
        alt: modifiers.alt(),
        ctrl: modifiers.ctrl(),
        shift: modifiers.shift(),
        mac_cmd: false,
        command: modifiers.ctrl(),
    }
}

fn egui_to_winit_cursor_icon(icon: CursorIcon) -> Option<winit::window::CursorIcon> {
    match icon {
        CursorIcon::None => None,

        CursorIcon::Default => Some(winit::window::CursorIcon::Default),
        CursorIcon::ContextMenu => Some(winit::window::CursorIcon::ContextMenu),
        CursorIcon::Help => Some(winit::window::CursorIcon::Help),
        CursorIcon::PointingHand => Some(winit::window::CursorIcon::Hand),
        CursorIcon::Progress => Some(winit::window::CursorIcon::Progress),
        CursorIcon::Wait => Some(winit::window::CursorIcon::Wait),
        CursorIcon::Cell => Some(winit::window::CursorIcon::Cell),
        CursorIcon::Crosshair => Some(winit::window::CursorIcon::Crosshair),
        CursorIcon::Text => Some(winit::window::CursorIcon::Text),
        CursorIcon::VerticalText => Some(winit::window::CursorIcon::VerticalText),
        CursorIcon::Alias => Some(winit::window::CursorIcon::Alias),
        CursorIcon::Copy => Some(winit::window::CursorIcon::Copy),
        CursorIcon::Move => Some(winit::window::CursorIcon::Move),
        CursorIcon::NoDrop => Some(winit::window::CursorIcon::NoDrop),
        CursorIcon::NotAllowed => Some(winit::window::CursorIcon::NotAllowed),
        CursorIcon::Grab => Some(winit::window::CursorIcon::Grab),
        CursorIcon::Grabbing => Some(winit::window::CursorIcon::Grabbing),
        CursorIcon::AllScroll => Some(winit::window::CursorIcon::AllScroll),
        CursorIcon::ResizeHorizontal => Some(winit::window::CursorIcon::EwResize),
        CursorIcon::ResizeNeSw => Some(winit::window::CursorIcon::NeswResize),
        CursorIcon::ResizeNwSe => Some(winit::window::CursorIcon::NwseResize),
        CursorIcon::ResizeVertical => Some(winit::window::CursorIcon::NsResize),
        CursorIcon::ResizeEast => Some(winit::window::CursorIcon::EResize),
        CursorIcon::ResizeSouthEast => Some(winit::window::CursorIcon::SeResize),
        CursorIcon::ResizeSouth => Some(winit::window::CursorIcon::SResize),
        CursorIcon::ResizeSouthWest => Some(winit::window::CursorIcon::SwResize),
        CursorIcon::ResizeWest => Some(winit::window::CursorIcon::WResize),
        CursorIcon::ResizeNorthWest => Some(winit::window::CursorIcon::NwResize),
        CursorIcon::ResizeNorth => Some(winit::window::CursorIcon::NResize),
        CursorIcon::ResizeNorthEast => Some(winit::window::CursorIcon::NeResize),
        CursorIcon::ResizeColumn => Some(winit::window::CursorIcon::ColResize),
        CursorIcon::ResizeRow => Some(winit::window::CursorIcon::RowResize),
        CursorIcon::ZoomIn => Some(winit::window::CursorIcon::ZoomIn),
        CursorIcon::ZoomOut => Some(winit::window::CursorIcon::ZoomOut),
    }
}
