use pyo3::{
    prelude::*,
    types::{PyDict, PyString},
};

pub fn start<R: resources::Provider>(resource_provider: R) {
    let py_lock = Python::acquire_gil();
    let py = py_lock.python();

    let engine_module = PyModule::new(py, "pyrite").expect("failed to initialise engine module");
    engine_binding::load_bindings(engine_module);
    inject_python_module(py, engine_module);

    let entry_path = "source/entry.py";
    let entry_source = resource_provider
        .read_to_string(entry_path)
        .expect("failed to load entry.py");

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
    use std::collections::HashMap;

    pub enum EngineMode {
        Client,
        Server,
    }

    pub enum BlendMode {
        Halves,
        Alternate,
    }

    pub struct Config {
        application_name: String,
        application_version: String,
        engine_mode: EngineMode,
        // base_grid_size: u32, // should probably calculate this from the smallest tile size
        window_width: u32,
        window_height: u32,
        blend_mode: BlendMode,
        tiles: HashMap<String, Tileset>, // the key should be the set name and not file name
    }

    pub struct Tileset {
        filename: String,
        horizontal_tiles: u32,
        vertical_tiles: u32,
        tile_names: Vec<String>,
    }

    pub struct Engine {
        config: Option<Config>,
    }

    impl Engine {
        pub fn run(&mut self, config: Config) -> bool {
            println!("Running engine!");
            false
        }
    }
}

mod engine_binding {
    use super::*;
    use engine::Engine;
    use pyo3::wrap_pyfunction;

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
        // write a macro to handle the boiler plate of each binding
        // bind!(m, foo)
        //
        // instead of
        // m.add_wrapped(wrap_pyfunction!(foo)).expect("error loading foo");
        // m.add_wrapped(wrap_pyfunction!(run))
        //     .expect("error loading run");

        bind!(m, run);
    }

    /// run(configuration)
    /// --
    /// run the engine life cycle
    #[pyfunction]
    fn run(config: PyObject) -> bool {
        let py = unsafe { Python::assume_gil_acquired() };

        let config: std::collections::HashMap<String, PyObject> = config
            .extract(py)
            .expect("Type error when reading the configuration structure");

        let applicaiton_name = config
            .get("application_name")
            .and_then(|py_value| py_value.extract::<String>(py).ok())
            .unwrap_or(String::from("default"));

        dbg!(applicaiton_name);

        let config = unimplemented!();

        engine!().run(config)
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
