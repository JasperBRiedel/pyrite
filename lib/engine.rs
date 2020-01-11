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
    // base_grid_size: u32, // should probably calculate this from the smallest tile size
    pub window_width: u32,
    pub window_height: u32,
    pub blend_mode: BlendMode,
    pub tiles: HashMap<String, Tileset>, // the key should be the set name and not file name
}

#[derive(Debug)]
pub struct Tileset {
    pub filename: String,
    pub horizontal_tiles: u32,
    pub vertical_tiles: u32,
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

        // give the cpu a break, should probably calculate this
        // value in the future to avoid a spiral of death with the time keeping
        thread::sleep(Duration::from_millis(5));

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
        self.graphics_context.as_mut().unwrap().swap_buffers();

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
    pub fn mouse_position(&mut self, camera: graphics::Camera) -> (i64, i64) {
        if let Some(context) = &self.graphics_context {
            self.platform
                .mouse_position(context.windowed_context.window().inner_size(), camera)
        } else {
            (0, 0)
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

    fn initialise(&mut self) {
        // initialise renderer
        let graphics_context =
            graphics::Context::new(self.config.as_ref().unwrap(), &self.platform);
        self.graphics_context = Some(graphics_context);

        // load tile sets into renderer

        // hook up input system
    }

    fn clean(&mut self) {}
}