mod audio;
mod binding;
mod engine;
mod graphics;
mod platform;
pub mod resources;

use pyo3::prelude::*;
use std::thread;
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

    // dynamic update loop, may run slower than the target rate, but never faster.
    // It's important the game logic takes delta time into consideration, due to the variability of
    // the time step delta.
    let target_delta_time = Duration::from_secs_f64(1. / 60.);
    let mut last_frame_time = Instant::now();
    while engine!().get_running() {
        // dispatch engine / platform events
        for event in engine!().poll_events() {
            binding::raise_event(py, entry_module, &event);
        }

        // calculate time since last frame, add it to the accumulator.
        let delta_time = last_frame_time.elapsed();
        last_frame_time = Instant::now();

        // pass this frames delta time to the binding for the python delta_time() function to get
        // its value. This value should only be set for the duration of the step event.
        binding::set_delta_time(delta_time.as_secs_f64());

        // Dispatch time step event with delta time
        binding::raise_event(
            py,
            entry_module,
            &engine::Event::Step {
                delta_time: delta_time.as_secs_f64(),
            },
        );

        // clear delta time before processing events that aren't logic steps
        binding::set_delta_time(0.);

        // Allow the renderer to present a new frame if needed.
        engine!().render();

        // if we still have remaining time before we reach our target rate, sleep.
        if delta_time < target_delta_time {
            let remaining_time = target_delta_time - delta_time;
            thread::sleep(remaining_time);
        }
    }

    pyrite_log!("Cleaning up pyrite engine resources");
    binding::destroy_engine();
}
