use std::env;
use std::fs;
use std::io;
use std::io::Write;

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
            let project_name = join_strings(&args, " ");

            match command.as_str() {
                "new" => new_command(project_name, project_path),
                "run" => run_command(project_name, project_path),
                "build" => build_command(project_name, project_path),
                _ => unreachable!(),
            }
        }
        _ => println!("Unknown command, type 'help' for a list of commands."),
    }
}

fn interactive_mode() -> bool {
    print!("pyrite> ");
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
        println!("Please enter a command, type 'help' for a list of commands.");
        return true;
    }

    evaluate_command(command_with_args);

    true
}

fn display_help() {
    println!(
        r#"Pyrite engine CLI tool v0.1.0

Commands:
    Create new project
    new <name>

    Run the game in development mode
    run <name>
    
    Create game executables ready for distribution
    build <name>
    
    Exit the interactive tool mode.
    exit
        "#
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

fn new_command(project_name: String, project_path: String) {
    if project_name.len() <= 0 {
        println!("Please provide a project name, type 'help' for a list of commands.");
        return;
    }

    println!("Creating project \"{}\"", project_name);

    let tool_exe = env::current_exe().expect("failed to locate pyrite executable");

    let tool_dir = tool_exe
        .parent()
        .expect("failed to extract pyrite directory");

    let project_dir = tool_dir.join("projects").join(project_path);

    if project_dir.exists() {
        println!("A project with that name already exists, type 'help' for a list of commands");
        return;
    }

    fs::create_dir_all(&project_dir).expect("failed to create project directory");

    let template = include_str!("entry_template.py").replace("APPLICATION_NAME", &project_name);
    let entry_file_path = project_dir.join("entry.py");
    let mut entry_file = fs::File::create(entry_file_path).expect("failed to create entry.py");
    write!(entry_file, "{}", template).expect("failed to write entry.py");

    println!("Project created at \"{}\"", project_dir.display());
}

fn run_command(project_name: String, project_path: String) {
    if project_name.len() <= 0 {
        println!("Please provide a project name, type 'help' for a list of commands.");
        return;
    }

    let tool_exe = env::current_exe().expect("failed to locate pyrite executable");

    let tool_dir = tool_exe
        .parent()
        .expect("failed to extract pyrite directory");

    let project_dir = tool_dir.join("projects").join(project_path.clone());

    if !project_dir.exists() {
        println!(
            "Failed to find project with name \"{}\", type 'help' for a list of commands",
            project_name
        );
        return;
    }

    println!("Running {} - \"{}\"", project_name, project_dir.display());

    let resources = pyrite::resources::FilesystemProvider::new(project_dir);
    pyrite::start(resources);
}

fn build_command(project_name: String, project_path: String) {
    println!("building project {} - {}", project_name, project_path);
}
