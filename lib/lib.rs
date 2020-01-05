use py::pyobject::ItemProtocol;
use rustpython_compiler as py_c;
use rustpython_vm as py;

pub fn start<R: resources::Provider>(resource_provider: R) {
    let python_vm_config = py::PySettings {
        debug: false,
        inspect: false,
        optimize: 0,
        no_user_site: true,
        no_site: true,
        ignore_environment: true,
        quiet: true,
        verbose: 1,
        dont_write_bytecode: true,
        path_list: Vec::new(),
        argv: Vec::new(),
    };

    let python_vm = py::VirtualMachine::new(python_vm_config);

    let entry_path = "source/entry.py";
    let entry_source = resource_provider
        .read_to_string(entry_path)
        .expect("failed to load entry.py");

    let scope = python_vm.new_scope_with_builtins();

    let code_obj = python_vm
        .compile(
            &entry_source,
            py_c::compile::Mode::Exec,
            entry_path.to_string(),
        )
        .expect("failed to compile entry.py");

    scope
        .globals
        .set_item(
            "__file__",
            python_vm.new_str(entry_path.to_string()),
            &python_vm,
        )
        .expect("failed to set __file__ for entry");

    let result = python_vm.run_code_obj(code_obj, scope);

    match result {
        Ok(_) => (),
        Err(e) => py::print_exception(&python_vm, &e),
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
