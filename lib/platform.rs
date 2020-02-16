use crate::engine;
use crate::graphics::Viewport;
use glutin::dpi::PhysicalSize;
use glutin::event::{
    ElementState, Event, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::platform::desktop::EventLoopExtDesktop;
#[cfg(target_os = "linux")]
use glutin::platform::unix::EventLoopExtUnix;
use std::collections::{HashMap, VecDeque};

pub struct Platform {
    pub events: Option<EventLoop<()>>,
    button_states: HashMap<String, ButtonState>,
    logical_mouse_position: (i32, i32),
    smooth_mouse_scroll_accumulator: (f32, f32),
    engine_event_queue: VecDeque<engine::Event>,
    pub close_requested: bool,
}

#[cfg(target_os = "linux")]
fn new_platform_eventloop() -> EventLoop<()> {
    EventLoop::new_x11().unwrap_or_else(|_| EventLoop::new())
}

#[cfg(not(target_os = "linux"))]
fn new_platform_eventloop() -> EventLoop<()> {
    EventLoop::new()
}

impl Platform {
    pub fn new() -> Self {
        // try and create an x11 event loop first, then fall back to glutins defaults.
        let events = Some(new_platform_eventloop());

        let button_states = HashMap::new();

        let engine_event_queue = VecDeque::new();

        Self {
            events,
            button_states,
            logical_mouse_position: (0, 0),
            smooth_mouse_scroll_accumulator: (0., 0.),
            engine_event_queue,
            close_requested: false,
        }
    }

    pub fn service(&mut self) {
        // We need to remove the events loop from self as we pass self into a closure passed to
        // run_return and this causes borrow checker issues. This isn't optimal, but it's better
        // than buffering all the events first like in commit d88f27c. It also allows as to react
        // to events immediately, so we can do things like rendering new frames as the window is
        // being resized.
        let mut events = self.events.take().expect("lost the platform event loop");

        events.run_return(|event, _, control_flow| {
            *control_flow = ControlFlow::Exit;
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(physical_framebuffer_size) => unsafe {
                        gl::Viewport(
                            0,
                            0,
                            physical_framebuffer_size.width as i32,
                            physical_framebuffer_size.height as i32,
                        );
                    },
                    WindowEvent::CloseRequested => {
                        self.close_requested = true;
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        // possible bug here with hi-dpi screens
                        self.logical_mouse_position = position.into();
                    }
                    WindowEvent::ReceivedCharacter(c) => {
                        self.engine_event_queue.push_back(engine::Event::Text {
                            text: c.to_string(),
                        });
                    }
                    WindowEvent::MouseWheel { delta, .. } => {
                        match delta {
                            MouseScrollDelta::LineDelta(x, y) => {
                                self.smooth_mouse_scroll_accumulator.0 += x as f32;
                                self.smooth_mouse_scroll_accumulator.1 += y as f32;
                            }
                            MouseScrollDelta::PixelDelta(delta) => {
                                self.smooth_mouse_scroll_accumulator.0 += (delta.x / 10.) as f32;
                                self.smooth_mouse_scroll_accumulator.1 += (delta.y / 10.) as f32;
                            }
                        };

                        let mut raise_event = false;

                        let delta_x = if self.smooth_mouse_scroll_accumulator.0.abs() >= 1.0 {
                            let delta = self.smooth_mouse_scroll_accumulator.0;
                            self.smooth_mouse_scroll_accumulator.0 = 0.;
                            raise_event = true;
                            delta
                        } else {
                            0.
                        };

                        let delta_y = if self.smooth_mouse_scroll_accumulator.1.abs() >= 1.0 {
                            let delta = self.smooth_mouse_scroll_accumulator.1;
                            self.smooth_mouse_scroll_accumulator.1 = 0.;
                            raise_event = true;
                            delta
                        } else {
                            0.
                        };

                        if raise_event {
                            let event = engine::Event::Scroll {
                                x: delta_x as i32,
                                y: delta_y as i32,
                            };

                            self.engine_event_queue.push_back(event);
                        }
                    }
                    WindowEvent::MouseInput { button, state, .. } => {
                        let (transition, state) = match state {
                            ElementState::Pressed => ("PRESSED".to_owned(), ButtonState::Down),
                            ElementState::Released => ("RELEASED".to_owned(), ButtonState::Up),
                        };

                        let (button_code, button_name) = match button {
                            MouseButton::Left => {
                                ("MOUSE_1".to_owned(), Some("MOUSE_LEFT".to_owned()))
                            }
                            MouseButton::Middle => {
                                ("MOUSE_2".to_owned(), Some("MOUSE_MIDDLE".to_owned()))
                            }
                            MouseButton::Right => {
                                ("MOUSE_3".to_owned(), Some("MOUSE_RIGHT".to_owned()))
                            }
                            MouseButton::Other(code) => (format!("MOUSE_{}", code), None),
                        };

                        self.button_states.insert(button_code.clone(), state);

                        let button_code_event = engine::Event::Button {
                            button: button_code,
                            transition: transition.clone(),
                        };

                        self.engine_event_queue.push_back(button_code_event);

                        if let Some(button_name) = button_name {
                            self.button_states.insert(button_name.clone(), state);

                            let button_name_event = engine::Event::Button {
                                button: button_name,
                                transition,
                            };

                            self.engine_event_queue.push_back(button_name_event);
                        }
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        let (transition, state) = match input.state {
                            ElementState::Pressed => ("PRESSED".to_owned(), ButtonState::Down),
                            ElementState::Released => ("RELEASED".to_owned(), ButtonState::Up),
                        };

                        let scancode_str = format!("K{}", input.scancode);

                        let last_state = self.button_states.insert(scancode_str.clone(), state);

                        let scancode_event = engine::Event::Button {
                            button: scancode_str,
                            transition: transition.clone(),
                        };

                        if last_state.is_some() && last_state.unwrap() != state {
                            self.engine_event_queue.push_back(scancode_event);
                        } else if last_state.is_none() {
                            self.engine_event_queue.push_back(scancode_event);
                        }

                        if let Some(virtual_key) = input.virtual_keycode {
                            let key_str = virtual_key_to_string_identifier(virtual_key);

                            let last_state = self.button_states.insert(key_str.clone(), state);

                            let named_event = engine::Event::Button {
                                button: key_str,
                                transition: transition,
                            };

                            if last_state.is_some() && last_state.unwrap() != state {
                                self.engine_event_queue.push_back(named_event);
                            } else if last_state.is_none() {
                                self.engine_event_queue.push_back(named_event);
                            }
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        });

        self.events = Some(events);
    }

    pub fn mouse_position(
        &mut self,
        window_size: PhysicalSize<u32>,
        viewport: Viewport,
    ) -> (i32, i32) {
        let normalised_mouse_position = (
            self.logical_mouse_position.0 as f32 / window_size.width as f32,
            self.logical_mouse_position.1 as f32 / window_size.height as f32,
        );

        let (viewport_width, viewport_height) = viewport.get_dimensions_f32();

        (
            (normalised_mouse_position.0 * viewport_width) as i32,
            (normalised_mouse_position.1 * viewport_height) as i32,
        )
    }

    pub fn button_down(&mut self, button: String) -> bool {
        button
            .split('+')
            .map(|key| key.to_uppercase())
            .all(|button| {
                self.button_states
                    .get(&button)
                    .map_or(false, |state| *state == ButtonState::Down)
            })
    }

    pub fn poll_events(&mut self) -> Vec<engine::Event> {
        self.engine_event_queue.drain(..).collect()
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum ButtonState {
    Down,
    Up,
}

fn virtual_key_to_string_identifier(virtual_key: VirtualKeyCode) -> String {
    match virtual_key {
        VirtualKeyCode::Key0 => "NUMBER0",
        VirtualKeyCode::Key1 => "NUMBER1",
        VirtualKeyCode::Key2 => "NUMBER2",
        VirtualKeyCode::Key3 => "NUMBER3",
        VirtualKeyCode::Key4 => "NUMBER4",
        VirtualKeyCode::Key5 => "NUMBER5",
        VirtualKeyCode::Key6 => "NUMBER6",
        VirtualKeyCode::Key7 => "NUMBER7",
        VirtualKeyCode::Key8 => "NUMBER8",
        VirtualKeyCode::Key9 => "NUMBER9",
        VirtualKeyCode::A => "A",
        VirtualKeyCode::B => "B",
        VirtualKeyCode::C => "C",
        VirtualKeyCode::D => "D",
        VirtualKeyCode::E => "E",
        VirtualKeyCode::F => "F",
        VirtualKeyCode::G => "G",
        VirtualKeyCode::H => "H",
        VirtualKeyCode::I => "I",
        VirtualKeyCode::J => "J",
        VirtualKeyCode::K => "K",
        VirtualKeyCode::L => "L",
        VirtualKeyCode::M => "M",
        VirtualKeyCode::N => "N",
        VirtualKeyCode::O => "O",
        VirtualKeyCode::P => "P",
        VirtualKeyCode::Q => "Q",
        VirtualKeyCode::R => "R",
        VirtualKeyCode::S => "S",
        VirtualKeyCode::T => "T",
        VirtualKeyCode::U => "U",
        VirtualKeyCode::V => "V",
        VirtualKeyCode::W => "W",
        VirtualKeyCode::X => "X",
        VirtualKeyCode::Y => "Y",
        VirtualKeyCode::Z => "Z",
        VirtualKeyCode::Escape => "ESCAPE",
        VirtualKeyCode::F1 => "F1",
        VirtualKeyCode::F2 => "F2",
        VirtualKeyCode::F3 => "F3",
        VirtualKeyCode::F4 => "F4",
        VirtualKeyCode::F5 => "F5",
        VirtualKeyCode::F6 => "F6",
        VirtualKeyCode::F7 => "F7",
        VirtualKeyCode::F8 => "F8",
        VirtualKeyCode::F9 => "F9",
        VirtualKeyCode::F10 => "F10",
        VirtualKeyCode::F11 => "F11",
        VirtualKeyCode::F12 => "F12",
        VirtualKeyCode::F13 => "F13",
        VirtualKeyCode::F14 => "F14",
        VirtualKeyCode::F15 => "F15",
        VirtualKeyCode::F16 => "F16",
        VirtualKeyCode::F17 => "F17",
        VirtualKeyCode::F18 => "F18",
        VirtualKeyCode::F19 => "F19",
        VirtualKeyCode::F20 => "F20",
        VirtualKeyCode::F21 => "F21",
        VirtualKeyCode::F22 => "F22",
        VirtualKeyCode::F23 => "F23",
        VirtualKeyCode::F24 => "F24",
        VirtualKeyCode::Snapshot => "SNAPSHOT",
        VirtualKeyCode::Scroll => "SCROLL",
        VirtualKeyCode::Pause => "PAUSE",
        VirtualKeyCode::Insert => "INSERT",
        VirtualKeyCode::Home => "HOME",
        VirtualKeyCode::Delete => "DELETE",
        VirtualKeyCode::End => "END",
        VirtualKeyCode::PageDown => "PAGE_DOWN",
        VirtualKeyCode::PageUp => "PAGE_UP",
        VirtualKeyCode::Left => "LEFT",
        VirtualKeyCode::Up => "UP",
        VirtualKeyCode::Right => "RIGHT",
        VirtualKeyCode::Down => "DOWN",
        VirtualKeyCode::Back => "BACK",
        VirtualKeyCode::Return => "RETURN",
        VirtualKeyCode::Space => "SPACE",
        VirtualKeyCode::Compose => "COMPOSE",
        VirtualKeyCode::Caret => "CARET",
        VirtualKeyCode::Numlock => "NUMLOCK",
        VirtualKeyCode::Numpad0 => "NUMPAD0",
        VirtualKeyCode::Numpad1 => "NUMPAD1",
        VirtualKeyCode::Numpad2 => "NUMPAD2",
        VirtualKeyCode::Numpad3 => "NUMPAD3",
        VirtualKeyCode::Numpad4 => "NUMPAD4",
        VirtualKeyCode::Numpad5 => "NUMPAD5",
        VirtualKeyCode::Numpad6 => "NUMPAD6",
        VirtualKeyCode::Numpad7 => "NUMPAD7",
        VirtualKeyCode::Numpad8 => "NUMPAD8",
        VirtualKeyCode::Numpad9 => "NUMPAD9",
        VirtualKeyCode::Backslash => "BACKSLASH",
        VirtualKeyCode::Calculator => "CALCULATOR",
        VirtualKeyCode::Capital => "CAPITAL",
        VirtualKeyCode::Colon => "COLON",
        VirtualKeyCode::Comma => "COMMA",
        VirtualKeyCode::Convert => "CONVERT",
        VirtualKeyCode::Decimal => "DECIMAL",
        VirtualKeyCode::Divide => "DIVIDE",
        VirtualKeyCode::Equals => "EQUALS",
        VirtualKeyCode::Grave => "GRAVE",
        VirtualKeyCode::Kana => "KANA",
        VirtualKeyCode::Kanji => "KANJI",
        VirtualKeyCode::LAlt => "LEFT_ALT",
        VirtualKeyCode::LBracket => "LEFT_BRACKET",
        VirtualKeyCode::LControl => "LEFT_CONTROL",
        VirtualKeyCode::LShift => "LEFT_SHIFT",
        VirtualKeyCode::LWin => "LEFT_SUPER",
        VirtualKeyCode::Mail => "MAIL",
        VirtualKeyCode::MediaSelect => "MEDIA_SELECT",
        VirtualKeyCode::MediaStop => "MEDIA_STOP",
        VirtualKeyCode::Minus => "MINUS",
        VirtualKeyCode::Multiply => "MULTIPLY",
        VirtualKeyCode::Mute => "MUTE",
        VirtualKeyCode::MyComputer => "MY_COMPUTER",
        VirtualKeyCode::NavigateForward => "NAV_FORWARD",
        VirtualKeyCode::NavigateBackward => "NAV_BACKWARD",
        VirtualKeyCode::NextTrack => "NEXT_TRACK",
        VirtualKeyCode::NoConvert => "NO_CONVERT",
        VirtualKeyCode::NumpadComma => "NUMPAD_COMMA",
        VirtualKeyCode::NumpadEnter => "NUMPAD_ENTER",
        VirtualKeyCode::NumpadEquals => "NUMPAD_EQUALS",
        VirtualKeyCode::Period => "PERIOD",
        VirtualKeyCode::PlayPause => "PLAY_PAUSE",
        VirtualKeyCode::Power => "POWER",
        VirtualKeyCode::PrevTrack => "PREV_TRACK",
        VirtualKeyCode::RAlt => "RIGHT_ALT",
        VirtualKeyCode::RBracket => "RIGHT_BRACKET",
        VirtualKeyCode::RControl => "RIGHT_CONTROL",
        VirtualKeyCode::RShift => "RIGHT_SHIFT",
        VirtualKeyCode::RWin => "RIGHT_SUPER",
        VirtualKeyCode::Semicolon => "SEMICOLON",
        VirtualKeyCode::Slash => "SLASH",
        VirtualKeyCode::Sleep => "SLEEP",
        VirtualKeyCode::Stop => "STOP",
        VirtualKeyCode::Subtract => "SUBTRACT",
        VirtualKeyCode::Tab => "TAB",
        VirtualKeyCode::Underline => "UNDERLINE",
        VirtualKeyCode::VolumeUp => "VOLUME_UP",
        VirtualKeyCode::VolumeDown => "VOLUME_DOWN",
        _ => "",
    }
    .to_owned()
}
