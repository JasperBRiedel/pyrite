mod binding;
mod engine;
mod graphics;
mod platform;
pub mod resources;

use pyo3::{prelude::*, types::PyDict};

#[macro_export]
macro_rules! pyrite_log {
    ($($arg:tt)*) => {
        println!("Pyrite > {}", format!($($arg)*));
    }
}

pub fn start<R: resources::Provider + 'static>(resource_provider: R) {
    pyrite_log!("Pyrite {}", env!("CARGO_PKG_VERSION"));
    pyrite_log!("Acquiring python environment lock");
    let py_lock = Python::acquire_gil();
    let py = py_lock.python();

    pyrite_log!("Loading game entry source file");
    let entry_path = "entry.py";
    let entry_source = resource_provider
        .read_to_string(entry_path)
        .expect("failed to load entry.py");

    pyrite_log!("Preparing pyrite engine instance");
    let resources = Box::new(resource_provider);
    let engine = engine::Engine::new(resources);
    binding::load_engine(engine);

    pyrite_log!("Injecting pyrite engine module");
    let engine_module = PyModule::new(py, "pyrite").expect("failed to initialise engine module");
    binding::load_bindings(engine_module);
    inject_python_module(py, engine_module);

    pyrite_log!("Injecting pyrite imports module");
    PyModule::from_code(py, include_str!("importer.py"), "importer.py", "importer")
        .expect("failed to create python resource importer hook");

    pyrite_log!("Loading entry module");
    match PyModule::from_code(py, &entry_source, entry_path, "entry") {
        Ok(entry) => {
            pyrite_log!("Invoking entry module __entry__() function");
            match entry.call0("__entry__") {
                Err(e) => {
                    pyrite_log!("An error occurred during runtime of the entry module");
                    e.print(py);
                }
                _ => pyrite_log!("Entry module exited gracefully"),
            }
        }
        Err(e) => {
            pyrite_log!("An error occurred while importing the entry module");
            e.print(py);
        }
    }

    pyrite_log!("Cleaning up pyrite engine resources");
    binding::destroy_engine();
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
