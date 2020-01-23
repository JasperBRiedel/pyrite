use crate::graphics;
use crate::platform::Platform;
use crate::resources;
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum EngineMode {
    Client,
    Server,
}

impl EngineMode {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "client" => Self::Client,
            "server" => Self::Server,
            _ => Self::Client,
        }
    }
}

#[derive(Debug)]
pub enum BlendMode {
    Halves,
    Alternate,
}

impl BlendMode {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "halves" => Self::Halves,
            "alternate" => Self::Alternate,
            _ => Self::Halves,
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub application_name: String,
    pub application_version: String,
    pub engine_mode: EngineMode,
    pub window_width: u32,
    pub window_height: u32,
    pub window_resizable: bool,
    pub viewport_width: i32,
    pub viewport_height: i32,
    pub blend_mode: BlendMode,
    pub tileset_width: u32,
    pub tileset_height: u32,
    pub tileset_path: String,
    pub tile_names: Vec<String>,
}

enum EngineState {
    Starting, // Initial engine state before any methods are called
    Loading,  // First iteration of the main loop will have this state
    Running,  // Running state set once the engine is initialised and run function invoked
    Exiting,  // Exiting set when the game requests that the engine exit
}

struct Timestep {
    accumulator: Duration,
    last_step: Instant,
}

impl Timestep {
    fn new() -> Self {
        Timestep {
            accumulator: Duration::from_secs(0),
            last_step: Instant::now(),
        }
    }

    fn step(&mut self, interval: f64) -> bool {
        let delta_time = 1. / interval;
        self.accumulator += self.last_step.elapsed();
        self.last_step = Instant::now();

        if self.accumulator.as_secs_f64() >= delta_time {
            self.accumulator -= Duration::from_secs_f64(delta_time);
            return true;
        }

        return false;
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    Button { button: String, transition: String },
    Scroll { x: i32, y: i32 },
    Text { text: String },
    Resized { width: i32, height: i32 },
}

pub struct Engine {
    config: Option<Config>,
    resources: Box<dyn resources::Provider>,
    state: EngineState,
    timesteps: HashMap<String, Timestep>,
    current_timestep_identifier: String,
    platform: Platform,
    graphics_context: Option<graphics::Context>,
}

impl Engine {
    pub fn new(resources: Box<dyn resources::Provider>) -> Self {
        Self {
            config: None,
            resources,
            state: EngineState::Starting,
            timesteps: HashMap::new(),
            current_timestep_identifier: String::from("outer"),
            graphics_context: None,
            platform: Platform::new(),
        }
    }

    // API Function
    pub fn run(&mut self, config: Config) -> bool {
        match self.state {
            EngineState::Loading => self.state = EngineState::Running,
            _ => (),
        }

        if self.config.is_none() {
            self.config = Some(dbg!(config));

            self.initialise();

            match self.state {
                EngineState::Starting => self.state = EngineState::Loading,
                _ => (),
            }
        }

        let frame_presented = self.graphics_context.as_mut().unwrap().present_frame();

        // The renderer optimises and will sometimes choose not to render or swap buffers, in this
        // case we will sleep the program for a moment if v-sync is enabled to give the cpu a
        // break.
        if !frame_presented {
            thread::sleep(Duration::from_millis(8));
        }

        if self.platform.close_requested {
            self.state = EngineState::Exiting;
        }

        match self.state {
            EngineState::Running | EngineState::Loading => true,
            EngineState::Exiting => {
                self.clean();
                false
            }
            _ => false,
        }
    }

    // API Function
    pub fn load(&mut self) -> bool {
        match self.state {
            EngineState::Loading => {
                self.state = EngineState::Running;
                true
            }
            _ => false,
        }
    }

    // API Function
    pub fn timestep(&mut self, timestep_identifier: String, interval: f64) -> bool {
        let timestep = self
            .timesteps
            .entry(timestep_identifier.clone())
            .or_insert(Timestep::new());

        self.platform.service(&mut self.graphics_context);

        let should_step = timestep.step(interval);

        if should_step {
            self.current_timestep_identifier = timestep_identifier;
        } else {
            if self.current_timestep_identifier != "outer" {
                self.current_timestep_identifier = String::from("outer");
            }
        }

        return should_step;
    }

    // API Function
    pub fn exit(&mut self) {
        self.state = EngineState::Exiting;
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
    pub fn set_viewport(&mut self, width: i32, height: i32) {
        if let Some(context) = &mut self.graphics_context {
            context.set_viewport(width, height);
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

    fn initialise(&mut self) {
        // initialise renderer
        let mut graphics_context =
            graphics::Context::new(self.config.as_ref().unwrap(), &self.platform);

        let config = self
            .config
            .as_ref()
            .expect("tried to initialise renderer without configuration being loaded first");

        // load tile sets into renderer
        graphics_context.load_tileset(&config, &self.resources);

        self.graphics_context = Some(graphics_context);
    }

    pub fn clean(&mut self) {
        self.graphics_context.take();
        self.platform.service(&mut None);
    }
}
