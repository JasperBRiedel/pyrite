use super::*;
use engine::*;
use pyo3::types::PyDict;
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
                None => {
                    panic!("An engine function was invoked without an available engine instance")
                }
            }
        }
    };
}

pub fn load_bindings(m: &PyModule) {
    bind!(m, run);
    bind!(m, load);
    bind!(m, timestep);
    bind!(m, exit);
    bind!(m, mouse_position);
    bind!(m, button_down);
    bind!(m, poll_events);
    bind!(m, camera);
}

pub fn destroy_engine() {
    unsafe {
        ENGINE_INSTANCE = None;
    }
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

/// mouse_position(camera) -> (x, y)
/// --
/// Return the x and y position of the mouse.
///
/// Needs to be provided with a camera to determine the coordinate space to be used
#[pyfunction]
fn mouse_position() -> (i64, i64) {
    engine!().mouse_position()
}

/// camera(viewport_width, viewport_height)
/// --
/// Set the camera viewport
#[pyfunction]
fn camera(viewport_width: f32, viewport_height: f32) {
    engine!().set_camera(viewport_width, viewport_height)
}

/// button_down(button) -> Boolean
/// --
/// returns true if button is down
#[pyfunction]
fn button_down(button: String) -> bool {
    engine!().button_down(button)
}

/// poll_events() -> [event]
/// --
/// returns an array of events
#[pyfunction]
fn poll_events() -> Vec<PyObject> {
    engine!()
        .poll_events()
        .into_iter()
        .map(|e| event_into_pyobject(e))
        .collect()
}

fn event_into_pyobject(event: Event) -> PyObject {
    let py = unsafe { Python::assume_gil_acquired() };

    let py_event = PyDict::new(py);

    match event {
        Event::Button { button, transition } => {
            py_event
                .set_item("type", "button")
                .expect("failed to set event item");
            py_event
                .set_item("button", button)
                .expect("failed to set event item");
            py_event
                .set_item("transition", transition)
                .expect("failed to set event item");
        }
        Event::Scroll { x, y } => {
            py_event
                .set_item("type", "scroll")
                .expect("failed to set event item");
            py_event.set_item("x", x).expect("failed to set event item");
            py_event.set_item("y", y).expect("failed to set event item");
        }
        Event::Text { text } => {
            py_event
                .set_item("type", "text")
                .expect("failed to set event item");
            py_event
                .set_item("text", text)
                .expect("failed to set event item");
        }
    };

    return py_event.to_object(py);
}

// fn pyobject_into_camera(camera: PyObject) -> graphics::Camera {
//     let py = unsafe { Python::assume_gil_acquired() };

//     let camera: HashMap<String, PyObject> = camera
//         .extract(py)
//         .expect("Type error when reading camera structure");

//     let viewport_width = extract_or!(py, camera, "viewport_width", f32, 10.);
//     let viewport_height = extract_or!(py, camera, "viewport_height", f32, 10.);

//     graphics::Camera {
//         viewport_width,
//         viewport_height,
//     }
// }

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

    let engine_mode_string = extract_or!(py, config, "engine_mode", String, "client".to_string());

    let engine_mode = EngineMode::from_string(&engine_mode_string);

    let window_width = extract_or!(py, config, "window_width", u32, 800);
    let window_height = extract_or!(py, config, "window_height", u32, 600);
    let viewport_width = extract_or!(py, config, "viewport_width", f32, 10.);
    let viewport_height = extract_or!(py, config, "viewport_height", f32, 10.);

    let blend_mode_string = extract_or!(py, config, "blend_mode", String, "halves".to_string());

    let blend_mode = BlendMode::from_string(&blend_mode_string);

    let tileset_width = extract_or!(py, config, "tileset_width", u32, 3);
    let tileset_height = extract_or!(py, config, "tileset_width", u32, 3);
    let tileset_path = extract_or!(py, config, "tileset_path", String, "default.png".to_owned());
    let tile_names = extract_or!(py, config, "tile_names", Vec<String>, Vec::new());

    Config {
        application_name,
        application_version,
        engine_mode,
        window_width,
        window_height,
        viewport_width,
        viewport_height,
        blend_mode,
        tileset_width,
        tileset_height,
        tileset_path,
        tile_names,
    }
}
