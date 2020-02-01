use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

use pyrite::pyrite_log;
use pyrite::resources;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() > 0 {
        evaluate_command(args);
        return;
    }

    display_help();

    while interactive_mode() {}
}

fn evaluate_command(mut command_with_args: Vec<String>) {
    let command = command_with_args.remove(0).to_lowercase();
    let args = command_with_args;

    match command.as_str() {
        "help" => display_help(),
        "new" | "run" | "build" => {
            let project_path = join_strings(&args, "-");
            let tool_exe = env::current_exe().expect("failed to locate pyrite executable");
            let tool_dir = tool_exe
                .parent()
                .expect("failed to extract pyrite directory");
            let project_dir = tool_dir.join("projects").join(&project_path);
            let project_name = join_strings(&args, " ");

            match command.as_str() {
                "new" => new_command(project_name, project_dir),
                "run" => run_command(project_name, project_dir),
                "build" => build_command(project_name, project_path, project_dir),
                _ => unreachable!(),
            }
        }
        _ => pyrite_log!("Unknown command, type 'help' for a list of commands."),
    }
}

fn interactive_mode() -> bool {
    print!("> ");
    io::stdout()
        .flush()
        .expect("failed to flush output before read");

    let mut command = String::new();
    io::stdin()
        .read_line(&mut command)
        .expect("failed to read command");
    let command_with_args: Vec<String> =
        command.split_whitespace().map(|s| s.to_string()).collect();

    if let Some(command) = command_with_args.get(0) {
        if command == "exit" {
            return false;
        }
    } else {
        pyrite_log!("Please enter a command, type 'help' for a list of commands.");
        return true;
    }

    evaluate_command(command_with_args);

    true
}

fn display_help() {
    println!(
        r#"Pyrite engine CLI tool {}

Commands:
    Create new project
    new <name>

    Run the game in development mode
    run <name>
    
    Create game executables ready for distribution
    build <name>
    
    Exit the interactive tool mode.
    exit
        "#,
        env!("CARGO_PKG_VERSION"),
    )
}

fn join_strings(strings: &Vec<String>, seperator: &str) -> String {
    // calculate total size of all strings
    let size = strings.iter().fold(0, |size, s| size + s.len());
    // calculate total size of strings with separators
    let size = size + (seperator.len() * size);

    let mut joined_string = String::with_capacity(size);

    for (i, s) in strings.iter().enumerate() {
        joined_string.push_str(s);
        if i < strings.len() - 1 {
            joined_string.push_str(seperator);
        }
    }

    joined_string
}

fn new_command(project_name: String, project_dir: PathBuf) {
    if project_name.len() <= 0 {
        pyrite_log!("Please provide a project name, type 'help' for a list of commands.");
        return;
    }

    if project_dir.exists() {
        pyrite_log!("A project with that name already exists, type 'help' for a list of commands");
        return;
    }

    fs::create_dir_all(&project_dir).expect("failed to create project directory");

    let entry_template =
        include_str!("../template/entry.py").replace("APPLICATION_NAME", &project_name);
    let entry_file_path = project_dir.join("entry.py");
    let mut entry_file = fs::File::create(entry_file_path).expect("failed to create entry.py");
    write!(entry_file, "{}", entry_template).expect("failed to write entry.py");

    let tileset_template = include_bytes!("../template/tiles.png");
    let tileset_file_path = project_dir.join("tiles.png");
    let mut entry_file = fs::File::create(tileset_file_path).expect("failed to create tiles.png");
    entry_file
        .write_all(tileset_template)
        .expect("failed to write tiles.png");

    pyrite_log!("Created project \"{}\"", project_name);
    pyrite_log!("{}", project_dir.display());
}

fn run_command(project_name: String, project_dir: PathBuf) {
    if project_name.len() <= 0 {
        pyrite_log!("Please provide a project name, type 'help' for a list of commands.");
        return;
    }

    if !project_dir.exists() {
        pyrite_log!(
            "Failed to find project with name \"{}\", type 'help' for a list of commands",
            project_name
        );
        return;
    }

    pyrite_log!("Running {}", project_name);
    pyrite_log!("{}", project_dir.display());

    let resources = pyrite::resources::FilesystemProvider::new(project_dir);
    pyrite::start(resources);
}

fn build_command(project_name: String, project_path: String, project_dir: PathBuf) {
    pyrite_log!("Building project {}", project_name,);
    pyrite_log!("{}", project_dir.display());

    // create resource package
    let packaged_bytes = if let Some(packaged_bytes) =
        resources::PackagedProvider::create_packaged_data(project_dir)
    {
        pyrite_log!("Resource package created");
        packaged_bytes
    } else {
        return;
    };

    pyrite_log!("Creating windows build");
    write_player_binary(
        &project_path,
        format!("{}-win.exe", project_path),
        include_bytes!("../template/player-windows.exe"),
        &packaged_bytes,
    );

    pyrite_log!("Creating linux build");
    write_player_binary(
        &project_path,
        format!("{}-linux", project_path),
        include_bytes!("../template/player-linux"),
        &packaged_bytes,
    );
}

fn write_player_binary(
    project_path: &str,
    binary_name: String,
    binary_bytes: &[u8],
    resources_bytes: &[u8],
) {
    if binary_bytes.len() <= 0 {
        pyrite_log!("This version of pyrite can't build game executables");
        pyrite_log!("Please visit the store page to purchase the full version");
        return;
    }

    let tool_exe = env::current_exe().expect("failed to locate pyrite executable");
    let tool_dir = tool_exe
        .parent()
        .expect("failed to extract pyrite directory");
    let builds_path = tool_dir.join("builds").join(project_path);
    fs::create_dir_all(&builds_path).expect("failed to create build directory");
    try_copy(
        &tool_dir.join("python38.zip"),
        &builds_path.join("python38.zip"),
    );
    try_copy(
        &tool_dir.join("python38.dll"),
        &builds_path.join("python38.dll"),
    );
    let player_binary_path = builds_path.join(&binary_name);

    let player_binary_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&player_binary_path);

    match player_binary_file {
        Ok(mut file) => {
            if let Err(e) = file.write_all(binary_bytes) {
                pyrite_log!(
                    "Failed to write to binary {} {}",
                    player_binary_path.display(),
                    e
                );
            }
            if let Err(e) = file.write_all(resources_bytes) {
                pyrite_log!(
                    "Failed to write resources {} {}",
                    player_binary_path.display(),
                    e
                );
            }
        }
        Err(e) => pyrite_log!(
            "Failed to open binary for writing {} {}",
            player_binary_path.display(),
            e
        ),
    }

    pyrite_log!("Created binary \"{}\"", binary_name);
    pyrite_log!("{}", player_binary_path.display());
}

fn try_copy(source: &Path, destination: &Path) {
    if let Err(_) = fs::copy(source, destination) {
        pyrite_log!("WARN > Failed to copy build file {}", source.display())
    }
}
