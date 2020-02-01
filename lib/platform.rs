use crate::engine;
use crate::graphics;
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

    pub fn service(&mut self, graphics_context: &mut Option<graphics::Context>) {
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
                            ElementState::Pressed => ("pressed".to_owned(), ButtonState::Down),
                            ElementState::Released => ("released".to_owned(), ButtonState::Up),
                        };

                        let (button_code, button_name) = match button {
                            MouseButton::Left => ("M1".to_owned(), Some("mouse_left".to_owned())),
                            MouseButton::Middle => {
                                ("M2".to_owned(), Some("mouse_middle".to_owned()))
                            }
                            MouseButton::Right => ("M3".to_owned(), Some("mouse_right".to_owned())),
                            MouseButton::Other(code) => (format!("M{}", code), None),
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
                            ElementState::Pressed => ("pressed".to_owned(), ButtonState::Down),
                            ElementState::Released => ("released".to_owned(), ButtonState::Up),
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
        match self.button_states.get(&button) {
            Some(state) => *state == ButtonState::Down,
            None => false,
        }
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
        VirtualKeyCode::Key0 => "number0",
        VirtualKeyCode::Key1 => "number1",
        VirtualKeyCode::Key2 => "number2",
        VirtualKeyCode::Key3 => "number3",
        VirtualKeyCode::Key4 => "number4",
        VirtualKeyCode::Key5 => "number5",
        VirtualKeyCode::Key6 => "number6",
        VirtualKeyCode::Key7 => "number7",
        VirtualKeyCode::Key8 => "number8",
        VirtualKeyCode::Key9 => "number9",
        VirtualKeyCode::A => "a",
        VirtualKeyCode::B => "b",
        VirtualKeyCode::C => "c",
        VirtualKeyCode::D => "d",
        VirtualKeyCode::E => "e",
        VirtualKeyCode::F => "f",
        VirtualKeyCode::G => "g",
        VirtualKeyCode::H => "h",
        VirtualKeyCode::I => "i",
        VirtualKeyCode::J => "j",
        VirtualKeyCode::K => "k",
        VirtualKeyCode::L => "l",
        VirtualKeyCode::M => "m",
        VirtualKeyCode::N => "n",
        VirtualKeyCode::O => "o",
        VirtualKeyCode::P => "p",
        VirtualKeyCode::Q => "q",
        VirtualKeyCode::R => "r",
        VirtualKeyCode::S => "s",
        VirtualKeyCode::T => "t",
        VirtualKeyCode::U => "u",
        VirtualKeyCode::V => "v",
        VirtualKeyCode::W => "w",
        VirtualKeyCode::X => "x",
        VirtualKeyCode::Y => "y",
        VirtualKeyCode::Z => "z",
        VirtualKeyCode::Escape => "escape",
        VirtualKeyCode::F1 => "f1",
        VirtualKeyCode::F2 => "f2",
        VirtualKeyCode::F3 => "f3",
        VirtualKeyCode::F4 => "f4",
        VirtualKeyCode::F5 => "f5",
        VirtualKeyCode::F6 => "f6",
        VirtualKeyCode::F7 => "f7",
        VirtualKeyCode::F8 => "f8",
        VirtualKeyCode::F9 => "f9",
        VirtualKeyCode::F10 => "f10",
        VirtualKeyCode::F11 => "f11",
        VirtualKeyCode::F12 => "f12",
        VirtualKeyCode::F13 => "f13",
        VirtualKeyCode::F14 => "f14",
        VirtualKeyCode::F15 => "f15",
        VirtualKeyCode::F16 => "f16",
        VirtualKeyCode::F17 => "f17",
        VirtualKeyCode::F18 => "f18",
        VirtualKeyCode::F19 => "f19",
        VirtualKeyCode::F20 => "f20",
        VirtualKeyCode::F21 => "f21",
        VirtualKeyCode::F22 => "f22",
        VirtualKeyCode::F23 => "f23",
        VirtualKeyCode::F24 => "f24",
        VirtualKeyCode::Snapshot => "snapshot",
        VirtualKeyCode::Scroll => "scroll",
        VirtualKeyCode::Pause => "pause",
        VirtualKeyCode::Insert => "insert",
        VirtualKeyCode::Home => "home",
        VirtualKeyCode::Delete => "delete",
        VirtualKeyCode::End => "end",
        VirtualKeyCode::PageDown => "page_down",
        VirtualKeyCode::PageUp => "page_up",
        VirtualKeyCode::Left => "left",
        VirtualKeyCode::Up => "up",
        VirtualKeyCode::Right => "right",
        VirtualKeyCode::Down => "down",
        VirtualKeyCode::Back => "back",
        VirtualKeyCode::Return => "return",
        VirtualKeyCode::Space => "space",
        VirtualKeyCode::Compose => "compose",
        VirtualKeyCode::Caret => "caret",
        VirtualKeyCode::Numlock => "numlock",
        VirtualKeyCode::Numpad0 => "numpad0",
        VirtualKeyCode::Numpad1 => "numpad1",
        VirtualKeyCode::Numpad2 => "numpad2",
        VirtualKeyCode::Numpad3 => "numpad3",
        VirtualKeyCode::Numpad4 => "numpad4",
        VirtualKeyCode::Numpad5 => "numpad5",
        VirtualKeyCode::Numpad6 => "numpad6",
        VirtualKeyCode::Numpad7 => "numpad7",
        VirtualKeyCode::Numpad8 => "numpad8",
        VirtualKeyCode::Numpad9 => "numpad9",
        VirtualKeyCode::Backslash => "backslash",
        VirtualKeyCode::Calculator => "calculator",
        VirtualKeyCode::Capital => "capital",
        VirtualKeyCode::Colon => "colon",
        VirtualKeyCode::Comma => "comma",
        VirtualKeyCode::Convert => "convert",
        VirtualKeyCode::Decimal => "decimal",
        VirtualKeyCode::Divide => "divide",
        VirtualKeyCode::Equals => "equals",
        VirtualKeyCode::Grave => "grave",
        VirtualKeyCode::Kana => "kana",
        VirtualKeyCode::Kanji => "kanji",
        VirtualKeyCode::LAlt => "left_alt",
        VirtualKeyCode::LBracket => "left_bracket",
        VirtualKeyCode::LControl => "left_control",
        VirtualKeyCode::LShift => "left_shift",
        VirtualKeyCode::LWin => "left_super",
        VirtualKeyCode::Mail => "mail",
        VirtualKeyCode::MediaSelect => "media_select",
        VirtualKeyCode::MediaStop => "media_stop",
        VirtualKeyCode::Minus => "minus",
        VirtualKeyCode::Multiply => "multiply",
        VirtualKeyCode::Mute => "mute",
        VirtualKeyCode::MyComputer => "my_computer",
        VirtualKeyCode::NavigateForward => "nav_forward",
        VirtualKeyCode::NavigateBackward => "nav_backward",
        VirtualKeyCode::NextTrack => "next_track",
        VirtualKeyCode::NoConvert => "no_convert",
        VirtualKeyCode::NumpadComma => "numpad_comma",
        VirtualKeyCode::NumpadEnter => "numpad_enter",
        VirtualKeyCode::NumpadEquals => "numpad_equals",
        VirtualKeyCode::Period => "period",
        VirtualKeyCode::PlayPause => "play_pause",
        VirtualKeyCode::Power => "power",
        VirtualKeyCode::PrevTrack => "prev_track",
        VirtualKeyCode::RAlt => "right_alt",
        VirtualKeyCode::RBracket => "right_bracket",
        VirtualKeyCode::RControl => "right_control",
        VirtualKeyCode::RShift => "right_shift",
        VirtualKeyCode::RWin => "right_super",
        VirtualKeyCode::Semicolon => "semicolon",
        VirtualKeyCode::Slash => "slash",
        VirtualKeyCode::Sleep => "sleep",
        VirtualKeyCode::Stop => "stop",
        VirtualKeyCode::Subtract => "subtract",
        VirtualKeyCode::Tab => "tab",
        VirtualKeyCode::Underline => "underline",
        VirtualKeyCode::VolumeUp => "volume_up",
        VirtualKeyCode::VolumeDown => "volume_down",
        _ => "",
    }
    .to_owned()
}
