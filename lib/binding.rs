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
fn mouse_position(camera: PyObject) -> (i64, i64) {
    let camera = pyobject_into_camera(camera);

    engine!().mouse_position(camera)
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
    let events = engine!().poll_events();
    unimplemented!()
}

fn pyobject_into_camera(camera: PyObject) -> graphics::Camera {
    let py = unsafe { Python::assume_gil_acquired() };

    let camera: HashMap<String, PyObject> = camera
        .extract(py)
        .expect("Type error when reading camera structure");

    let x = extract_or!(py, camera, "x", i64, 0);
    let y = extract_or!(py, camera, "y", i64, 0);
    let z = extract_or!(py, camera, "z", i64, 0);
    let width = extract_or!(py, camera, "width", i64, 10);
    let height = extract_or!(py, camera, "height", i64, 10);

    graphics::Camera {
        x,
        y,
        z,
        width,
        height,
    }
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

    let engine_mode_string = extract_or!(py, config, "engine_mode", String, "client".to_string());

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
