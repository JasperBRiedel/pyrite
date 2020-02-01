use crate::pyrite_log;
use std::fs;
use std::io::Read;
use std::path::PathBuf;

pub trait Provider {
    fn read_to_string(&self, path: &str) -> Option<String>;

    fn read_to_bytes(&self, path: &str) -> Option<Vec<u8>>;

    fn exists(&self, path: &str) -> bool;
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

    fn exists(&self, path: &str) -> bool {
        self.root_path.join(path).exists()
    }
}

pub struct PackagedProvider {}

impl PackagedProvider {
    pub fn create_packaged_data(root_path: PathBuf) -> Option<Vec<u8>> {
        pyrite_log!("Starting resource packager...");

        if !root_path.is_dir() {
            pyrite_log!("Resource directory expected: {}", root_path.display());
            return None;
        }

        // try the access the resources directory, and create an iterator of top level files.
        let resource_files = if let Ok(resources_dir) = root_path.read_dir() {
            resources_dir
                .filter_map(|entry| entry.ok().map(|e| e.path()))
                .filter(|path| path.is_file())
                .filter_map(|file_path| {
                    file_path
                        .file_name()
                        .and_then(|os_str| os_str.to_owned().into_string().ok())
                        .map(|file_name| (file_path, file_name))
                })
        } else {
            pyrite_log!("Resource directory expected: {}", root_path.display());
            return None;
        };

        for (resource_path, resource_name) in resource_files {}

        return None;
    }
}
