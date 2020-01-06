use py::pyobject::ItemProtocol;
use py::pyobject::PyObjectRef;
use py::pyobject::PyResult;
use py::pyobject::PyValue;
use rustpython_compiler as py_c;
use rustpython_vm as py;

pub fn start<R: resources::Provider>(resource_provider: R) {
    let python_vm = py::VirtualMachine::new(py::PySettings::default());

    load_engine_module(&python_vm);

    let entry_path = "source/entry.py";
    let entry_source = resource_provider
        .read_to_string(entry_path)
        .expect("failed to load entry.py");

    let scope = python_vm.new_scope_with_builtins();

    if let Err(e) = dbg!(python_vm.import("pyrite", &[], 0)) {
        py::print_exception(&python_vm, &e);
    }

    let code_obj = python_vm
        .compile(
            &entry_source,
            py_c::compile::Mode::Exec,
            entry_path.to_string(),
        )
        .expect("failed to compile entry.py");

    let result = python_vm.run_code_obj(code_obj, scope);

    match result {
        Ok(_) => (),
        Err(e) => py::print_exception(&python_vm, &e),
    }
}

fn load_engine_module(vm: &py::VirtualMachine) {
    vm.stdlib_inits
        .try_borrow_mut()
        .expect("failed to load engine module")
        .insert("pyrite".to_string(), Box::new(engine_module_loader));
}

fn engine_module_loader(vm: &py::VirtualMachine) -> PyObjectRef {
    py::py_module!(vm, "pyrite", {
        "foo" => vm.context().new_rustfunc(foo),
    })
}

fn foo(vm: &py::VirtualMachine, args: py::function::PyFuncArgs) -> PyResult {
    println!("Foo called by python!");
    Ok(vm.get_none())
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
