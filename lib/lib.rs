mod binding;
mod engine;
mod graphics;
mod platform;
pub mod resources;

use pyo3::{prelude::*, types::PyDict};
use std::thread;
use std::time::Duration;

pub fn start<R: resources::Provider + 'static>(resource_provider: R) {
    let py_lock = Python::acquire_gil();
    let py = py_lock.python();

    let entry_path = "entry.py";
    let entry_source = resource_provider
        .read_to_string(entry_path)
        .expect("failed to load entry.py");

    let resources = Box::new(resource_provider);
    let engine = engine::Engine::new(resources);
    binding::load_engine(engine);

    let engine_module = PyModule::new(py, "pyrite").expect("failed to initialise engine module");
    binding::load_bindings(engine_module);
    inject_python_module(py, engine_module);

    PyModule::from_code(py, include_str!("importer.py"), "importer.py", "importer")
        .expect("failed to create python resource importer hook");

    match PyModule::from_code(py, &entry_source, entry_path, "entry") {
        Ok(_) => binding::destroy_engine(), // game exited gracefully, clean up and exit engine.
        Err(e) => {
            binding::destroy_engine();
            println!("An error occurred in the python environment:");
            e.print(py); // game syntax or logic error occurred, write crash log, clean up and exit engine.
        }
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
