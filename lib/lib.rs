use pyo3::{
    prelude::*,
    types::{PyDict, PyString},
};

pub fn start<R: resources::Provider + 'static>(resource_provider: R) {
    let py_lock = Python::acquire_gil();
    let py = py_lock.python();

    let entry_path = "source/entry.py";
    let entry_source = resource_provider
        .read_to_string(entry_path)
        .expect("failed to load entry.py");

    let resources = Box::new(resource_provider);
    let engine = engine::Engine::new(resources);
    engine_binding::load_engine(engine);

    let engine_module = PyModule::new(py, "pyrite").expect("failed to initialise engine module");
    engine_binding::load_bindings(engine_module);
    inject_python_module(py, engine_module);

    match PyModule::from_code(py, &entry_source, entry_path, "entry") {
        Ok(_) => (),           // game exited gracefully, clean up and exit engine.
        Err(e) => e.print(py), // game syntax or logic error occurred, write crash log, clean up and exit engine.
    }
}

fn inject_python_module(py: Python, module: &PyModule) {
    py.import("sys")
        .expect("failed to import python sys module")
        .dict()
        .get_item("modules")
        .expect("failed to get python modules dictionary")
        .downcast_mut::<PyDict>()
        .expect("failed to turn sys.modules into a PyDict")
        .set_item(module.name().expect("module missing name"), module)
        .expect("failed to inject module");
}

mod engine {
    use crate::resources;
    use std::collections::HashMap;
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

    pub struct Engine {
        config: Option<Config>,
        resources: Box<dyn resources::Provider>,
        state: EngineState,
        timesteps: HashMap<String, Timestep>,
    }

    impl Engine {
        pub fn new(resources: Box<dyn resources::Provider>) -> Self {
            Self {
                config: None,
                resources,
                state: EngineState::Starting,
                timesteps: HashMap::new(),
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
        pub fn timestep(&mut self, label: String, interval: f64) -> bool {
            let timestep = self.timesteps.entry(label).or_insert(Timestep::new());

            // poll input system

            timestep.step(interval)
        }

        // API Function
        pub fn exit(&mut self) {
            self.state = EngineState::Exiting;
        }

        fn initialise(&mut self) {
            // initialise renderer

            // load tile sets into renderer

            // hook up input system
        }

        fn clean(&mut self) {}
    }
}

mod engine_binding {
    use super::*;
    use engine::*;
    use pyo3::wrap_pyfunction;
    use std::collections::HashMap;

    static mut ENGINE_INSTANCE: Option<Engine> = None;

    pub fn load_engine(e: Engine) {
        unsafe {
            ENGINE_INSTANCE = Some(e);
        }
    }

    macro_rules! bind {
        ($module:ident, $func:ident) => {
            $module
                .add_wrapped(wrap_pyfunction!($func))
                .expect(format!("failed to load binding for {}", stringify!($func)).as_str());
        };
    }

    macro_rules! engine {
        () => {
            unsafe {
                match &mut ENGINE_INSTANCE {
                    Some(e) => e,
                    None => panic!(
                        "An engine function was invoked without an available engine instance"
                    ),
                }
            }
        };
    }

    pub fn load_bindings(m: &PyModule) {
        bind!(m, run);
        bind!(m, load);
        bind!(m, timestep);
        bind!(m, exit);
    }

    macro_rules! extract_or {
        ($py:ident, $map:ident, $key:literal, $type:ty, $default:expr) => {
            $map.get($key)
                .and_then(|py_object| py_object.extract::<$type>($py).ok())
                .unwrap_or($default)
        };
    }

    /// run(configuration)
    /// --
    /// run the engine life cycle
    #[pyfunction]
    fn run(config: PyObject) -> bool {
        let config = pyobject_into_configuration(config);
        engine!().run(config)
    }

    /// load()
    /// --
    /// returns true when the game should load
    #[pyfunction]
    fn load() -> bool {
        engine!().load()
    }

    /// timestep(interval)
    /// --
    /// Return true interval amount of seconds
    #[pyfunction]
    fn timestep(label: String, interval: f64) -> bool {
        engine!().timestep(label, interval)
    }

    /// exit()
    /// --
    /// Initiate engine shut down
    #[pyfunction]
    fn exit() {
        engine!().exit();
    }

    fn pyobject_into_configuration(config: PyObject) -> Config {
        let py = unsafe { Python::assume_gil_acquired() };

        let config: HashMap<String, PyObject> = config
            .extract(py)
            .expect("Type error when reading the configuration structure");

        let application_name = extract_or!(
            py,
            config,
            "application_name",
            String,
            "default".to_string()
        );

        let application_version = extract_or!(
            py,
            config,
            "application_version",
            String,
            "0.0.0".to_string()
        );

        let engine_mode_string =
            extract_or!(py, config, "engine_mode", String, "client".to_string());

        let engine_mode = EngineMode::from_string(&engine_mode_string);

        let window_width = extract_or!(py, config, "window_width", u32, 800);
        let window_height = extract_or!(py, config, "window_height", u32, 600);

        let blend_mode_string = extract_or!(py, config, "blend_mode", String, "halves".to_string());

        let blend_mode = BlendMode::from_string(&blend_mode_string);

        let tiles = pyobject_into_tiles(
            extract_or!(py, config, "tiles", HashMap<String, PyObject>, HashMap::new()),
        );

        Config {
            application_name,
            application_version,
            engine_mode,
            window_width,
            window_height,
            blend_mode,
            tiles,
        }
    }

    fn pyobject_into_tiles(py_tiles: HashMap<String, PyObject>) -> HashMap<String, Tileset> {
        let py = unsafe { Python::assume_gil_acquired() };

        let mut tiles = HashMap::with_capacity(py_tiles.len());

        for (name, py_tileset) in py_tiles {
            let py_tileset = py_tileset
                .extract::<HashMap<String, PyObject>>(py)
                .expect("Type error reading tile set configuration");

            let filename = extract_or!(py, py_tileset, "filename", String, String::new());
            let horizontal_tiles = extract_or!(py, py_tileset, "horizontal_tiles", u32, 1);
            let vertical_tiles = extract_or!(py, py_tileset, "vertical_tiles", u32, 1);
            let tile_names = extract_or!(py, py_tileset, "tile_names", Vec<String>, Vec::new());

            let tileset = Tileset {
                filename,
                horizontal_tiles,
                vertical_tiles,
                tile_names,
            };

            tiles.insert(name, tileset);
        }

        tiles
    }
}

pub mod resources {
    use std::fs;
    use std::io::Read;
    use std::path::{Path, PathBuf};

    pub trait Provider {
        fn read_to_string(&self, path: &str) -> Option<String>;

        fn read_to_bytes(&self, path: &str) -> Option<Vec<u8>>;
    }

    pub struct FilesystemProvider {
        root_path: PathBuf,
    }

    impl FilesystemProvider {
        pub fn new(root_path: PathBuf) -> Self {
            FilesystemProvider { root_path }
        }
    }

    impl Provider for FilesystemProvider {
        fn read_to_string(&self, path: &str) -> Option<String> {
            let file_path = self.root_path.join(path);
            let mut file = fs::File::open(file_path).ok()?;
            let mut string_data = String::new();
            file.read_to_string(&mut string_data).ok()?;
            Some(string_data)
        }

        fn read_to_bytes(&self, path: &str) -> Option<Vec<u8>> {
            let file_path = self.root_path.join(path);
            let mut file = fs::File::open(file_path).ok()?;
            let mut data = Vec::new();
            file.read_to_end(&mut data).ok()?;
            Some(data)
        }
    }
}
