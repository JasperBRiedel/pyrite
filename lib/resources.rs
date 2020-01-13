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
        println!("reading resource: {}", path);
        let file_path = self.root_path.join(path);
        let mut file = fs::File::open(file_path).ok()?;
        let mut string_data = String::new();
        file.read_to_string(&mut string_data).ok()?;
        Some(string_data)
    }

    fn read_to_bytes(&self, path: &str) -> Option<Vec<u8>> {
        println!("reading resource: {}", path);
        let file_path = self.root_path.join(path);
        let mut file = fs::File::open(file_path).ok()?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).ok()?;
        Some(data)
    }
}
