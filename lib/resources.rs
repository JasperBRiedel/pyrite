use crate::pyrite_log;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
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

pub struct PackagedProvider {
    resource_index: HashMap<String, Vec<u8>>,
}

impl Provider for PackagedProvider {
    fn read_to_string(&self, path: &str) -> Option<String> {
        self.read_to_bytes(path)
            .and_then(|bytes| String::from_utf8(bytes).ok())
    }

    fn read_to_bytes(&self, path: &str) -> Option<Vec<u8>> {
        self.resource_index.get(path).cloned()
    }

    fn exists(&self, path: &str) -> bool {
        self.resource_index.contains_key(path)
    }
}

impl PackagedProvider {
    pub fn new() -> Self {
        let package_path = dbg!(env::current_exe().expect("failed to locate pyrite executable"));
        // useful for testing. Loads the resource package of another project.
        // use std::str::FromStr;
        // let package_path =
        //     String::from("/home/jasper/projects/rust/pyrite/target/debug/builds/packaged-linux")
        //         .into();

        let mut resource_index = HashMap::new();

        let mut shared_binary =
            fs::File::open(&package_path).expect("failed to open binary resources");

        // discover resource offset
        shared_binary
            .seek(SeekFrom::End(-8))
            .expect("failed to seek to resources offset location");
        let mut resources_offset_bytes = [0u8; 8];
        shared_binary
            .read_exact(&mut resources_offset_bytes)
            .expect("failed to read resource offset");
        let resources_offset = u64::from_be_bytes(resources_offset_bytes);

        // discover resource count
        shared_binary
            .seek(SeekFrom::End(-12))
            .expect("failed to seek to resources count location");
        let mut resource_count_bytes = [0u8; 4];
        shared_binary
            .read_exact(&mut resource_count_bytes)
            .expect("failed to read resource count");
        let resource_count = u32::from_be_bytes(resource_count_bytes);

        // seek backward to the start of the resources section
        shared_binary
            .seek(SeekFrom::End(-(resources_offset as i64)))
            .expect("failed to seek to resources start offset");

        // walk resources and set-up index table
        for _ in 0..resource_count {
            // read name length
            let mut name_length_bytes = [0u8; 4];
            shared_binary
                .read_exact(&mut name_length_bytes)
                .expect("failed to read name length");
            let name_length = u32::from_be_bytes(name_length_bytes);

            dbg!(name_length);

            // read resource name
            let mut name_bytes = Vec::new();
            shared_binary
                .by_ref()
                .take(name_length as u64)
                .read_to_end(&mut name_bytes)
                .expect("failed to read resource name");
            let resource_name =
                String::from_utf8(name_bytes).expect("failed to decode resource name");

            dbg!(&resource_name);

            // read resource length
            let mut resource_length_bytes = [0u8; 8];
            shared_binary
                .read_exact(&mut resource_length_bytes)
                .expect("failed to read name length");
            let resource_length = u64::from_be_bytes(resource_length_bytes);

            dbg!(resource_length);

            // read resource
            let mut resource_bytes = Vec::new();
            shared_binary
                .by_ref()
                .take(resource_length)
                .read_to_end(&mut resource_bytes)
                .expect("failed to read resource name");

            dbg!(resource_bytes.len());

            resource_index.insert(resource_name, resource_bytes);
        }

        Self { resource_index }
    }

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

        // package_data has the following repeating structure
        // resource_name_length: u32
        // resource_name: resource_name_length
        // resource_length: u64
        // resource_data: resource_length
        // ..
        // resource_count: u32
        // resource_package_len: u64
        let mut package_data = Vec::new();
        let mut resource_count: u32 = 0;

        for (resource_path, resource_name) in resource_files {
            if let Ok(mut resource_file) = File::open(resource_path) {
                let mut resource_data = Vec::new();
                match resource_file.read_to_end(&mut resource_data) {
                    Ok(bytes_read) => pyrite_log!("Packaging {} {}b", resource_name, bytes_read),
                    Err(e) => {
                        pyrite_log!("Failed {} {}", resource_name, e);
                        return None;
                    }
                }
                let resource_data_length: u64 = resource_data.len() as u64;
                let mut resource_name_data = resource_name.as_bytes().to_vec();
                let resource_name_data_length: u32 = resource_name_data.len() as u32;

                package_data.extend_from_slice(&resource_name_data_length.to_be_bytes());
                package_data.append(&mut resource_name_data);
                package_data.extend_from_slice(&resource_data_length.to_be_bytes());
                package_data.append(&mut resource_data);

                resource_count += 1;
            }
        }

        // add 12 bytes to offset the resource_package_len and resource_count bytes.
        let package_data_length: u64 = package_data.len() as u64 + 12;

        package_data.extend_from_slice(&resource_count.to_be_bytes());
        package_data.extend_from_slice(&package_data_length.to_be_bytes());

        pyrite_log!("Total: {}", package_data_length);

        return Some(package_data);
    }
}
