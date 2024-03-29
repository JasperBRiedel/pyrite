use crate::audio;
use crate::graphics;
use crate::platform::Platform;
use crate::pyrite_log;
use crate::resources;

#[derive(Debug)]
pub struct Config {
    pub application_name: String,
    pub application_version: String,
    pub viewport_scale: i32,
    pub viewport_width: i32,
    pub viewport_height: i32,
    pub tileset_width: u32,
    pub tileset_height: u32,
    pub tileset_path: String,
    pub tile_names: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum Event {
    Load,
    Button { button: String, transition: String },
    Scroll { x: i32, y: i32 },
    Text { text: String },
    Step { delta_time: f64 },
    Exit,
}

impl Event {
    pub fn type_str(&self) -> &str {
        match self {
            Self::Load => "LOAD",
            Self::Button { .. } => "BUTTON",
            Self::Scroll { .. } => "SCROLL",
            Self::Text { .. } => "TEXT",
            Self::Step { .. } => "STEP",
            Self::Exit => "EXIT",
        }
    }
}

pub struct Engine {
    config: Option<Config>,
    resources: Box<dyn resources::Provider>,
    platform: Platform,
    graphics_context: Option<graphics::Context>,
    audio: audio::AudioServer,
    running: bool,
}

impl Engine {
    pub fn new(resources: Box<dyn resources::Provider>) -> Self {
        Self {
            config: None,
            resources,
            platform: Platform::new(),
            graphics_context: None,
            audio: audio::AudioServer::new(),
            running: true,
        }
    }

    pub fn get_running(&self) -> bool {
        self.running && !self.platform.close_requested
    }

    pub fn load_configuration(&mut self, config: Config) {
        if self.config.is_none() {
            pyrite_log!("Loading configuration");
            log_config(&config);
            self.config = Some(config);

            let graphics_context = graphics::Context::new(
                self.config.as_ref().unwrap(),
                &self.platform,
                &self.resources,
            );

            self.graphics_context = Some(graphics_context);
        }
    }

    pub fn render(&mut self) -> bool {
        let frame_presented = self.graphics_context.as_mut().unwrap().present_frame();
        // The renderer optimises and will sometimes choose not to render or swap buffers.
        // return the value for the game or binding to decide on the best course of action in this
        // case.
        return frame_presented;
    }

    // API Function
    pub fn exit(&mut self) {
        pyrite_log!("Exit requested");
        self.running = false;
    }

    // API Function
    pub fn mouse_position(&mut self) -> (i32, i32) {
        if let Some(context) = &self.graphics_context {
            self.platform.mouse_position(
                context.windowed_context.window().inner_size(),
                context.get_viewport().clone(),
            )
        } else {
            (0, 0)
        }
    }

    // API Function
    pub fn set_viewport(&mut self, width: i32, height: i32, scale: i32) {
        if let Some(context) = &mut self.graphics_context {
            context.set_viewport(width, height, scale);
        }
    }

    // API function
    pub fn clear_tiles(&mut self) {
        let context = match &mut self.graphics_context {
            Some(c) => c,
            _ => return,
        };

        let (viewport_width, viewport_height) = context.get_viewport().get_dimensions();

        for x in 0..viewport_width {
            for y in 0..viewport_height {
                context.set_tile(
                    (x, y),
                    "none",
                    (0, 0, 0),
                    (false, false),
                    "none",
                    (0, 0, 0),
                    (false, false),
                );
            }
        }
    }

    // API Function
    pub fn set_tile(
        &mut self,
        position: (i32, i32),
        front_tile: String,
        front_color: (u8, u8, u8),
        front_flip: (bool, bool),
        back_tile: String,
        back_color: (u8, u8, u8),
        back_flip: (bool, bool),
    ) {
        if let Some(context) = self.graphics_context.as_mut() {
            context.set_tile(
                position,
                &front_tile,
                front_color,
                front_flip,
                &back_tile,
                back_color,
                back_flip,
            );
        }
    }

    // API Function
    pub fn button_down(&mut self, button: String) -> bool {
        self.platform.button_down(button)
    }

    // API Function
    pub fn poll_events(&mut self) -> Vec<Event> {
        self.platform.service();
        // eventually will inject other events here such as network api stuff
        self.platform.poll_events()
    }

    // API Function
    pub fn resource_read(&mut self, path: String) -> String {
        self.resources
            .read_to_string(&path)
            .unwrap_or(String::new())
    }

    // API Function
    pub fn resource_exists(&self, path: String) -> bool {
        self.resources.exists(&path)
    }

    // API Function
    pub fn play_audio(&mut self, path: String) {
        self.audio.play(&path, &self.resources);
    }

    // API Function
    pub fn stop_audio(&mut self, path: String) {
        if path == "*" {
            self.audio.stop_all();
        } else {
            self.audio.stop(&path);
        }
    }

    // API Function
    pub fn pause_audio(&mut self, path: String) {
        self.audio.pause(&path);
    }

    // API Function
    pub fn volume_audio(&mut self, path: String, value: f32) {
        self.audio.volume(&path, value);
    }

    pub fn clean(&mut self) {
        self.graphics_context.take();
        self.platform.service();
    }
}

fn log_config(config: &Config) {
    macro_rules! log_config_item {
        ($config:ident, $item:ident) => {
            pyrite_log!("{}: {:?}", stringify!($item), $config.$item);
        };
    }

    log_config_item!(config, application_name);
    log_config_item!(config, application_version);
    log_config_item!(config, viewport_scale);
    log_config_item!(config, viewport_width);
    log_config_item!(config, viewport_height);
    log_config_item!(config, tileset_width);
    log_config_item!(config, tileset_height);
    log_config_item!(config, tileset_path);
    log_config_item!(config, tile_names);
}
