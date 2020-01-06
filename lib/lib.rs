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

    pub struct Engine {
        config: Option<Config>,
        resources: Box<dyn resources::Provider>,
    }

    impl Engine {
        pub fn new(resources: Box<dyn resources::Provider>) -> Self {
            Self {
                config: None,
                resources,
            }
        }

        pub fn run(&mut self, config: Config) -> bool {
            if self.config.is_none() {
                println!("Loading configuration...");
                self.config = Some(dbg!(config));
            }

            println!("Running engine!");
            false
        }
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
                .expect(stringify!(failed to load binding for $func));
        }
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
