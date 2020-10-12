#![allow(clippy::pedantic)]

mod editors;
mod language;
mod project;

use crate::{editors::*, language::*, project::*};
use std::{
    env,
    ffi::OsStr,
    fs::{create_dir_all, read_dir, remove_dir_all, remove_file, rename, File},
    io::prelude::*,
    path::Path,
    process::Command,
};

fn check_for_toml() -> Result<(), String> {
    if Path::new("./Ocean.toml").exists() {
        return Ok(());
    }
    Err("Could not find Ocean.toml, please make sure that you are in a valid project directory.".to_string())
}

fn help(argument: Option<&String>) {
    if argument.is_some() {
        println!("Command \"{}\" not found.", argument.unwrap());
    }
    println!(
        "
Usage: ocean [OPTION]

Create and manage C and C++ projects.

    build           Builds the current project
    clean           Cleans the current project's build artifacts
    get             Returns the values set in the Ocean.toml
    set             Sets the values in side Ocean.toml
    help, --help    Shows this help text
    new             Creates a new C/C++ project in a new directory
    run             Runs the current project, builds if no build is present
        "
    );
}

fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

fn build(args: &[String], project: &mut Project) -> Result<(), String> {
    check_for_toml()?;

    let mut build_mode = "debug";
    let mut is_verbose = false;

    let executable_name = {
        if cfg!(windows) {
            format!("{}.exe", project.get_name())
        } else {
            project.get_name().clone()
        }
        .replace(" ", "_")
    };

    for directory in project.get_directories().get_all_dirs() {
        if !Path::new(directory).exists() {
            match create_dir_all(directory) {
                Err(e) => println!("Could not create directory \"{}\": {}", directory, e),
                _ => continue,
            }
        }
    }

    let file_extension = project.get_language().get_extension();
    let compiler = project.get_compiler().get_compiler_command(project.get_language());

    if !args.is_empty() {
        match args[0].as_str() {
            "--help" => {
                println!(
                    "
Usage: ocean build [OPTIONS]

By default, this builds projects in debug mode.

Options:
    -d, --debug     Builds the current project in debug mode (this is turned on by default)
    -r, --release   Builds the current project in release mode
    -v, --verbose   Makes the compiler output verbose.
            "
                );
                return Ok(());
            }
            "-r" | "--release" => build_mode = "release",
            "-d" | "--debug" => build_mode = "debug",
            "-v" | "--verbose" => is_verbose = true,
            _ => (),
        }
    }

    let build_path = format!("{}/{}", project.get_directories().get_build_dir(), build_mode);
    let object_path = format!("{}/{}", project.get_directories().get_objects_dir(), build_mode);

    let mut compilable = vec![];

    let source_files = read_dir(project.get_directories().get_source_dir().to_string()).unwrap();
    for file in source_files {
        let file_name = file.unwrap().path().clone();

        if get_extension_from_filename(file_name.to_str().unwrap()).unwrap() == file_extension {
            compilable.push(file_name);
        }
    }

    let flags = match build_mode {
        "release" => "-Wall -Wextra -O3",
        _ => "-Wall -Wextra -Og",
    };

    if !Path::new(&object_path).exists() {
        if let Err(e) = create_dir_all(object_path.clone()) {
            println!("Could not create directory \"{}\": {}", object_path, e)
        }
    }

    if !Path::new(&build_path).exists() {
        if let Err(e) = create_dir_all(build_path.clone()) {
            println!("Could not create directory \"{}\": {}", build_path, e)
        }
    }

    let mut object_files = vec![];

    for file in compilable {
        println!(
            "Compiling {} to {}.o...",
            file.file_name().unwrap().to_str().unwrap(),
            file.file_stem().unwrap().to_str().unwrap()
        );

        let mut c = Command::new(compiler.clone());

        if is_verbose {
            c.arg("-v");
        }

        c.arg("-c")
            .arg(file.to_str().unwrap())
            .spawn()
            .expect("Could not execute compiler")
            .wait()
            .unwrap();

        rename(
            format!("{}.o", file.file_stem().unwrap().to_str().unwrap()),
            format!("{}/{}.o", object_path, file.file_stem().unwrap().to_str().unwrap()),
        )
        .expect("Could not move object file");

        object_files.push(format!(
            "{}/{}.o",
            object_path,
            file.file_stem().unwrap().to_str().unwrap()
        ));
    }

    let mut c = Command::new(compiler);

    if is_verbose {
        c.arg("-v");
    }

    for obj in object_files {
        c.arg(obj);
    }

    c.args(&flags.split(' ').collect::<Vec<&str>>())
        .arg("-o")
        .arg(format!("{}/{}", build_path, executable_name));

    for library_directory in project.get_library_dirs() {
        c.arg(format!("-L{}", library_directory));
    }

    for library in project.get_libraries() {
        c.arg(format!("-l{}", library));
    }

    c.spawn()
        .expect("Could not compile objects to final executable")
        .wait()
        .unwrap();

    Ok(())
}

fn run(args: &[String], project: &mut Project) -> Result<(), String> {
    let mut build_mode = "debug";

    if !args.is_empty() {
        match args[0].as_str() {
            "--help" => {
                println!(
                    "
Usage: ocean run [OPTIONS]

By default, this run projects in debug mode.

Options:
    -d, --debug     Runs the current project in debug mode (this is turned on by default)
    -r, --release   Runs the current project in release mode
    -v, --verbose   Makes the compiler output verbose.
            "
                );
                return Ok(());
            }
            "-r" | "--release" => build_mode = "release",
            "-d" | "--debug" => build_mode = "debug",
            // -v is handled by build()
            _ => (),
        }
    }

    build(args, project)?;

    let build_path = format!("{}/{}", project.get_directories().get_build_dir(), build_mode);

    let executable_name = {
        if cfg!(windows) {
            format!("{}.exe", project.get_name())
        } else {
            project.get_name().clone()
        }
        .replace(" ", "_")
    };

    let executable_path = format!("{}/{}", build_path, executable_name);

    if Path::new(&executable_path).exists() {
        Command::new(format!("./{}", executable_path))
            .spawn()
            .expect("Could not start application")
            .wait()
            .expect("Application exited unexpectedly");
    }

    Ok(())
}

fn new(args: &[String], project: &mut Project) -> Result<(), String> {
    let mut do_ccls = false;
    let mut do_vscode = false;

    if !args.is_empty() {
        match args[0].as_str() {
            "--help" => {
                println!(
                    "
Usage: ocean new [NAME] [OPTIONS]

This creates a new project with a generated Ocean.toml in a new directory with a specified NAME.
Options:
    -C                  Creates a new C project (default).
    -CXX                Creates a new C++ project.
    -b, --build-dir     Sets the build directory (default is \"./build\").
    -s, --source-dir    Sets the source directory (default is \"./src\").
    -o, --obj-dir       Sets the objects directory (default is \"./obj\").
    -c, --compiler      Sets the compiler for the current project (default is gcc for C and g++ for C++).
    --ccls              Outputs a .ccls file to be used with ccls. Allows a language server to be used with an editor \
                     like Vim, for example.
    --vscode            Outputs Visual Studio Code config files to make writing C/C++ easier.
            "
                );
                return Ok(());
            }
            _ => {
                let name = args[0].to_string();
                if name != "" {
                    project.set_name(name);
                } else {
                    return Err("Did not specify project name".to_string());
                }
            }
        }
    } else {
        return Err("Did not specify project name".to_string());
    }

    if Path::new(&format!("{}/Ocean.toml", project.get_name())).exists() {
        return Err("Cannot create a new project, Ocean.toml already exists in this directory.".to_string());
    }

    if Path::new(&format!("{}/", project.get_name())).exists()
        && read_dir(&format!("{}/", project.get_name())).unwrap().next().is_some()
    {
        return Err("Cannot create a new project, directory is not empty".to_string());
    }

    for index in 0..args[1..].len() {
        let lang = *project.get_language();
        match args[index + 1].as_str() {
            "-C" => project.set_language(Language::C),
            "-CXX" => project.set_language(Language::CXX),
            "-b" | "--build-dir" => project
                .get_directories_mut()
                .set_build_dir(args.get(index + 2).expect("Did not specify a build directory").clone()),
            "-s" | "--source-dir" => project
                .get_directories_mut()
                .set_source_dir(args.get(index + 2).expect("Did not specify a source directory").clone()),
            "-o" | "--obj-dir" => project.get_directories_mut().set_objects_dir(
                args.get(index + 2)
                    .expect("Did not specify an objects directory")
                    .clone(),
            ),
            "-c" | "--compiler" => project.get_compiler_mut().set_compiler_command(
                lang,
                args.get(index + 2)
                    .unwrap_or_else(|| panic!("Did not specify custom {} compiler", lang)),
            ),
            "--ccls" => do_ccls = true,
            "--vscode" => do_vscode = true,
            _ => (),
        }
    }

    let toml_content = toml::to_string(project).expect("Could not transform project data into Ocean.toml");
    let code_content = match project.get_language() {
        Language::C => {
            "#include <stdio.h>

int main() {
    printf(\"Hello, world\\n\");
}
"
        }
        Language::CXX => {
            "#include <iostream>

int main() {
    std::cout << \"Hello, world\" << std::endl;
}
"
        }
    };
    let ignore_content = "/build/\n/obj/";

    create_dir_all(&format!("{}/src", project.get_name())).expect("Could not create project and source directory");
    let mut file = File::create(&format!("{}/Ocean.toml", project.get_name())).expect("Could not create Ocean.toml");
    file.write_all(toml_content.as_bytes())
        .expect("Could not write to Ocean.toml");

    let mut code_file = File::create(&format!(
        "{}/{}/main.{}",
        project.get_name(),
        project.get_directories().get_source_dir(),
        project.get_language().get_extension()
    ))
    .expect("Could not create code file.");

    code_file
        .write_all(code_content.as_bytes())
        .unwrap_or_else(|_| panic!("Could not write to main.{}", project.get_language().get_extension()));

    let mut ignore_file =
        File::create(&format!("{}/.gitignore", project.get_name())).expect("Could not create .gitignore");
    ignore_file
        .write_all(ignore_content.as_bytes())
        .expect("Could not write into .gitignore");

    if do_ccls {
        // TODO
        let _ = CCLS::new();
        let _ = File::create(&format!("{}/.ccls", project.get_name())).expect("Could not create .ccls");
    }

    if do_vscode {
        let vscode = VsCode::new(project);
        let config_dir = vscode.get_config_dir();
        let configs = vscode.get_config();
        create_dir_all(config_dir.clone()).expect("Could not create .vscode directory");

        let mut properties = File::create(format!("{}/c_cpp_properties.json", config_dir,))
            .expect("Could not create c_cpp_properties.json");

        properties
            .write_all(
                configs
                    .get("c_cpp_properties")
                    .expect("Could not find c_cpp_properties in hashmap.")
                    .as_bytes(),
            )
            .expect("Could not write to c_cpp_properties.json");

        let mut launch = File::create(format!("{}/launch.json", config_dir)).expect("Could not write lauch.json");

        launch
            .write_all(
                configs
                    .get("launch")
                    .expect("Could not find launch in hashmap")
                    .as_bytes(),
            )
            .expect("Could not write to tasks.json");

        let mut tasks = File::create(format!("{}/tasks.json", config_dir)).expect("Could not write tasks.json");

        tasks
            .write_all(
                configs
                    .get("tasks")
                    .expect("Could not find tasks in hashmap")
                    .as_bytes(),
            )
            .expect("Could not write to tasks.json");
    }

    println!(
        "Created a new {} project \"{}\"",
        project.get_language(),
        project.get_name()
    );

    Ok(())
}

fn clean(project: &Project) -> Result<(), String> {
    check_for_toml()?;

    for directory in project.get_directories().get_all_dirs() {
        if directory == project.get_directories().get_source_dir() {
            continue;
        }

        remove_dir_all(directory).unwrap_or_else(|_| panic!("Cannot delete {}", directory));
    }

    Ok(())
}

fn set_data(args: &[String], project: &mut Project) -> Result<(), String> {
    check_for_toml()?;

    let help = "
Usage: ocean set [KEY]

This set values inside the Ocean project file to a value specified by the user.

Option:
    build_dir [DIRECTORY]                               Sets the build directory for the project.
    c++_compiler [COMPILER], cxx_compiler [COMPILER]    Set the compiler being used for the C++ project.
    c_compiler [COMPILER]                               Sets the compiler being used for the C project.
    compiler [COMPILER], current_compiler [COMPILER]    Sets the current compiler being used for the project.
    lang [LANG], language [LANG]                        Set the current language of the project.
    lib_dirs [DIRS], library_directories [DIRS]         Sets the library directories that would be searched by the \
                linker, split by commas.
    libs [LIBS], libraries [LIBS]                       Sets the libraries being compiled with the project, split by \
                commas.
    name [NAME]                                         Sets the name of the project.
    object_dir [DIRECTORY]                              Sets the object output directory.
    source_dir [DIRECTORY]                              Sets the source code directory.
    ";

    if args.is_empty() {
        println!("{}", help);
        return Err("No key given".to_string());
    } else if args[0] == "--help" {
        println!("{}", help);
        return Ok(());
    } else if args.len() > 2 {
        return Err("No value supplied for given key".to_string());
    }

    match (args[0].as_str(), &args[1]) {
        ("name", n) => project.set_name(n.clone()),
        (c, lang) if c == "lang" || c == "language" => match lang.to_lowercase().as_str() {
            "c++" | "cxx" => project.set_language(Language::CXX),
            "c" => project.set_language(Language::C),
            _ => return Err("Invalid language.".to_string()),
        },
        (c, libs) if c == "libs" || c == "libraries" => {
            for lib in libs.split(',') {
                project.add_library(lib.to_string());
            }
        }
        (c, dirs) if c == "lib_dirs" || c == "library_directories" => {
            for dir in dirs.split(',') {
                project.add_library_directories(dir.to_string());
            }
        }
        ("c_compiler", compiler) => project.set_compiler(Language::C, compiler.clone()),
        (c, compiler) if c == "c++_compiler" || c == "cxx_compiler" => {
            project.set_compiler(Language::CXX, compiler.clone())
        }
        (c, compiler) if c == "compiler" || c == "current_compiler" => project.set_current_compiler(compiler.clone()),
        ("object_dir", dir) => project.get_directories_mut().set_objects_dir(dir.clone()),
        ("source_dir", dir) => project.get_directories_mut().set_source_dir(dir.clone()),
        ("build_dir", dir) => project.get_directories_mut().set_build_dir(dir.clone()),
        _ => return Err("Incorrect data key.".to_string()),
    }

    remove_file("./Ocean.toml").expect("Couldn't delete Ocean.toml");
    let mut file = File::create("./Ocean.toml").expect("Couldn't open Ocean.toml");
    let toml_content = toml::to_string(project).expect("Could not transform project data into Ocean.toml");
    file.write_all(toml_content.as_bytes())
        .expect("Could not write to Ocean.toml");

    Ok(())
}

fn get_data(args: &[String], project: &Project) -> Result<(), String> {
    check_for_toml()?;

    let help = "
Usage: ocean get [KEY]

This gets the current values inside the Ocean project file related to a data key entered by the user.

Option:
    build_dir                       Prints the build directory for the current project.
    c++_compiler, cxx_compiler      Prints the compiler being used for the C++ project.
    c_compiler                      Prints the compiler being used for the C project.
    compiler, current_compiler      Prints the current compiler being used for the project.
    lang, language                  Prints the current language of the project.
    lib_dirs, library_directories   Prints the library directories that would be searched by the linker.
    libs, libraries                 Prints the libraries being compiled with the project.
    name                            Prints the name of the project.
    object_dir                      Prints the object output directory.
    source_dir                      Prints the source code directory.
    ";

    if args.is_empty() {
        println!("{}", help);
        return Err("No value given".to_string());
    }
    let data = args[0].as_str();
    match data {
        "--help" => println!("{}", help),
        "name" => println!("{}", project.get_name().clone()),
        "lang" | "language" => println!("{}", project.get_language().to_string()),
        "libs" | "libraries" => println!("{:#?}", project.get_libraries()),
        "lib_dirs" | "library_directories" => println!("{:#?}", project.get_directories().get_all_dirs()),
        "compiler" | "current_compiler" => println!(
            "{}",
            project
                .get_compiler()
                .get_compiler_command(project.get_language())
                .clone()
        ),
        "c_compiler" => println!("{}", project.get_compiler().get_compiler_command(&Language::C).clone()),
        "c++_compiler" | "cxx_compiler" => println!(
            "{}",
            project.get_compiler().get_compiler_command(&Language::CXX).clone()
        ),
        _ => eprintln!("Cannot find data key. Use --help to get help for this command."),
    };
    Ok(())
}

fn main() -> Result<(), String> {
    let mut args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        args = args[1..].to_vec();
    } else {
        help(None);
        return Err("No arguments were specified".to_string());
    }

    let mut project = {
        let mut contents = String::from("");

        if let Ok(mut f) = File::open("Ocean.toml") {
            if f.read_to_string(&mut contents).is_err() {
                return Err("Could not read file".to_string());
            }
        }

        toml::from_str(contents.as_str()).unwrap_or_default()
    };

    match args[0].as_str() {
        "build" => build(&args[1..], &mut project)?,
        "clean" => clean(&project)?,
        "get" => get_data(&args[1..], &project)?,
        "help" | "--help" => help(None),
        "new" => new(&args[1..], &mut project)?,
        "run" => run(&args[1..], &mut project)?,
        "set" => set_data(&args[1..], &mut project)?,
        _ => help(Some(&args[0])),
    };
    Ok(())
}
