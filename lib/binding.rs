use super::*;
use engine::*;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;
use std::collections::HashMap;

pub static mut ENGINE_INSTANCE: Option<Engine> = None;
static mut GAME_DATA: Option<&PyDict> = None;
static mut CURRENT_DELTA_TIME: f64 = 0.0;

macro_rules! bind {
    ($module:ident, $func:ident) => {
        $module
            .add_wrapped(wrap_pyfunction!($func))
            .expect(format!("failed to load binding for {}", stringify!($func)).as_str());
    };
}

#[macro_export]
macro_rules! engine {
    () => {
        unsafe {
            match &mut $crate::binding::ENGINE_INSTANCE {
                Some(e) => e,
                None => {
                    panic!("An engine function was invoked without an available engine instance")
                }
            }
        }
    };
}

macro_rules! extract_or {
    ($py:ident, $map:ident, $key:literal, $type:ty, $default:expr) => {
        $map.get($key)
            .and_then(|py_object| py_object.extract::<$type>($py).ok())
            .unwrap_or($default)
    };
}

pub fn set_delta_time(delta_time: f64) {
    unsafe {
        CURRENT_DELTA_TIME = delta_time;
    }
}

pub fn inject_engine(py: Python, engine: Engine) {
    // set engine instance to be called by python module functions.
    unsafe {
        ENGINE_INSTANCE = Some(engine);
        GAME_DATA = Some(PyDict::new(Python::assume_gil_acquired()));
    }

    // create python engine module and bind functions
    let engine_module = PyModule::new(py, "pyrite").expect("failed to initialise engine module");
    bind!(engine_module, game_data);
    bind!(engine_module, exit);
    bind!(engine_module, delta_time);
    bind!(engine_module, mouse_position);
    bind!(engine_module, button_down);
    bind!(engine_module, set_viewport);
    bind!(engine_module, set_tile);
    bind!(engine_module, resource_read);
    bind!(engine_module, resource_exists);

    // Inject the engine module into the python importer
    py.import("sys")
        .expect("failed to import python sys module")
        .dict()
        .get_item("modules")
        .expect("failed to get python modules dictionary")
        .downcast_mut::<PyDict>()
        .expect("failed to turn sys.modules into a PyDict")
        .set_item(
            engine_module.name().expect("module missing name"),
            engine_module,
        )
        .expect("failed to inject module");
}

pub fn destroy_engine() {
    engine!().clean();

    unsafe {
        ENGINE_INSTANCE = None;
        GAME_DATA = None;
    }
}

pub fn raise_event(py: Python, entry_module: &PyModule, event: &Event) {
    let event_type = event.type_str();
    let event_data = event_data_into_pyobject(&event);

    let event_result = entry_module.call1("__event__", (event_type, event_data));

    match event_result {
        Ok(_) => (),
        Err(e) => {
            // can probably capture game errors here
            pyrite_log!(
                "An error occurred in the game module while processing a {} event:",
                event_type
            );
            e.print(py);
        }
    }
}

pub fn get_configuration(entry_module: &PyModule) -> Option<Config> {
    let py_config = entry_module.call0("__config__").ok()?;

    let config: HashMap<String, PyObject> = py_config
        .extract()
        .expect("Type error when reading the configuration structure");

    let py = unsafe { Python::assume_gil_acquired() };

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

    let viewport_scale = extract_or!(py, config, "viewport_scale", i32, 2);
    let viewport_width = extract_or!(py, config, "viewport_width", i32, 10);
    let viewport_height = extract_or!(py, config, "viewport_height", i32, 10);

    let tileset_width = extract_or!(py, config, "tileset_width", u32, 3);
    let tileset_height = extract_or!(py, config, "tileset_height", u32, 3);
    let tileset_path = extract_or!(py, config, "tileset_path", String, "default.png".to_owned());
    let tile_names = extract_or!(py, config, "tile_names", Vec<String>, Vec::new());

    Some(Config {
        application_name,
        application_version,
        viewport_scale,
        viewport_width,
        viewport_height,
        tileset_width,
        tileset_height,
        tileset_path,
        tile_names,
    })
}

/// game_data()
/// --
/// Return a reference to the global game data dictionary
#[pyfunction]
fn game_data() -> &'static PyDict {
    unsafe { GAME_DATA.expect("Game data was accessed before initialised") }
}

/// exit()
/// --
/// Initiate engine shut down
#[pyfunction]
fn exit() {
    engine!().exit();
}

/// delta_time() -> dt
/// --
/// Return the time since the last frame
#[pyfunction]
fn delta_time() -> f64 {
    unsafe { CURRENT_DELTA_TIME }
}

/// mouse_position(camera) -> (x, y)
/// --
/// Return the x and y position of the mouse.
///
/// Needs to be provided with a camera to determine the coordinate space to be used
#[pyfunction]
fn mouse_position() -> (i32, i32) {
    engine!().mouse_position()
}

/// set_viewport(viewport_width, viewport_height)
/// --
/// Set the viewport in tiles
#[pyfunction]
fn set_viewport(viewport_width: i32, viewport_height: i32, viewport_scale: i32) {
    engine!().set_viewport(viewport_width, viewport_height, viewport_scale)
}

/// set_tile(name, x, y)
/// set_tile(name, x, y, r, g, b)
/// set_tile(name, x, y, r, g, b, flip_x, flip_y)
/// --
/// Add a tile to the scene
#[pyfunction]
fn set_tile(
    position: (i32, i32),
    front_tile: String,
    front_color: (u8, u8, u8),
    front_flip: (bool, bool),
    back_tile: Option<String>,
    back_color: Option<(u8, u8, u8)>,
    back_flip: Option<(bool, bool)>,
) {
    let back_tile = back_tile.unwrap_or_else(|| "none".to_owned());
    let back_color = back_color.unwrap_or((0, 0, 0));
    let back_flip = back_flip.unwrap_or((false, false));

    engine!().set_tile(
        position,
        front_tile,
        front_color,
        front_flip,
        back_tile,
        back_color,
        back_flip,
    );
}

/// button_down(button) -> Boolean
/// --
/// returns true if button is down
#[pyfunction]
fn button_down(button: String) -> bool {
    engine!().button_down(button)
}

/// resource_read(path)
/// --
/// Read in the contents of a resource file
#[pyfunction]
fn resource_read(path: String) -> String {
    engine!().resource_read(path)
}

/// resource_exists(path)
/// --
/// Check if a resource exists
#[pyfunction]
fn resource_exists(path: String) -> bool {
    engine!().resource_exists(path)
}

fn event_data_into_pyobject(event: &Event) -> PyObject {
    let py = unsafe { Python::assume_gil_acquired() };

    let py_event = PyDict::new(py);

    match event {
        Event::Load => (),
        Event::Button { button, transition } => {
            py_event
                .set_item("button", button)
                .expect("failed to set event item");
            py_event
                .set_item("transition", transition)
                .expect("failed to set event item");
        }
        Event::Scroll { x, y } => {
            py_event.set_item("x", x).expect("failed to set event item");
            py_event.set_item("y", y).expect("failed to set event item");
        }
        Event::Text { text } => {
            py_event
                .set_item("text", text)
                .expect("failed to set event item");
        }
        Event::Step { delta_time } => {
            py_event
                .set_item("delta_time", delta_time)
                .expect("failed to set event item");
        }
    };

    return py_event.to_object(py);
}
