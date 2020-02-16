mod binding;
mod engine;
mod graphics;
mod platform;
pub mod resources;

use pyo3::{prelude::*, types::PyDict};
use std::time::{Duration, Instant};

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

    pyrite_log!("Building pyrite engine instance");
    let resources = Box::new(resource_provider);
    let engine = engine::Engine::new(resources);
    pyrite_log!("Building python bindings");
    binding::inject_engine(py, engine);

    pyrite_log!("Injecting pyrite imports module");
    PyModule::from_code(py, include_str!("importer.py"), "importer.py", "importer")
        .expect("failed to create python resource importer hook");

    pyrite_log!("Loading entry module");
    let entry_module = match PyModule::from_code(py, &entry_source, entry_path, "entry") {
        Ok(module) => module,
        Err(e) => {
            pyrite_log!("An error occurred while importing the entry module");
            e.print(py);
            return;
        }
    };

    // load configuration via callback.
    match binding::get_configuration(&entry_module) {
        Some(config) => engine!().load_configuration(config),
        None => {
            pyrite_log!("Failed to get configuration from __config__ in entry module");
            return;
        }
    }

    // instruct game logic to load
    binding::raise_event(py, entry_module, &engine::Event::Load);

    // time keeping for 60hz fixed update logic loop
    let delta_time = Duration::from_secs_f64(1. / 60.);
    let mut current_time = Instant::now();
    let mut accumulated_time = Duration::from_secs(0);

    // while running
    while engine!().get_running() {
        // calculate time since last frame, add it to the accumulator.
        accumulated_time += current_time.elapsed();
        current_time = Instant::now();

        for event in engine!().poll_events() {
            binding::raise_event(py, entry_module, &event);
        }

        while accumulated_time >= delta_time {
            accumulated_time -= delta_time;
            binding::raise_event(
                py,
                entry_module,
                &engine::Event::Step {
                    delta_time: delta_time.as_secs_f64(),
                },
            );
        }

        engine!().render();
    }

    pyrite_log!("Cleaning up pyrite engine resources");
    binding::destroy_engine();
}
