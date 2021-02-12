#[cfg(feature = "git")]
use crate::git;
use crate::{
    cache::Cache, common::StrRet, editors::*, language::*, platform::*, project::*,
};
use std::{
    env::{self, current_dir, set_current_dir},
    ffi::OsStr,
    fs::{create_dir_all, read_dir, remove_dir_all, remove_file, rename, File},
    io::prelude::*,
    path::{Path, PathBuf},
    process::Command,
};

pub struct Commands;

impl Commands {
    fn pretty_toml(toml_content: String) -> String {
        let mut split: Vec<&str> = toml_content.split('\n').collect();

        for i in (0..split.len() - 1).rev() {
            let below = *match split.get(i + 1) {
                Some(v) => v,
                None => continue,
            };

            if !split[i].is_empty() && below.starts_with('[') {
                split.insert(i + 1, "")
            }
        }

        split.join("\n")
    }

    fn get_toml(
        path: Option<&str>,
        search_count: Option<u32>,
    ) -> Result<Project, StrRet> {
        let search_count = search_count.unwrap_or(0);

        if search_count > 4 {
            return Err(
                "Could not find Ocean.toml, please make sure that you are in a valid \
                 project directory."
                    .into(),
            );
        }

        let cwd = current_dir().unwrap();
        let path = path.unwrap_or_else(|| cwd.to_str().unwrap());

        if Path::new(format!("{}/Ocean.toml", path).as_str()).exists() {
            set_current_dir(path).unwrap();
            return Ok({
                let mut contents = String::from("");

                if let Ok(mut f) = File::open("Ocean.toml") {
                    if f.read_to_string(&mut contents).is_err() {
                        return Err("Could not read file".into());
                    }
                };

                toml::from_str(contents.as_str()).unwrap()
            });
        }

        Self::get_toml(
            Some(format!("{}/..", path).as_str()),
            Some(search_count + 1),
        )
    }

    pub fn help(argument: Option<&String>) {
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

    pub fn get_extension_from_filename(filename: &str) -> Option<&str> {
        Path::new(filename).extension().and_then(OsStr::to_str)
    }

    fn build_file(
        project: &Project,
        binary: &mut Binary,
        build_mode: &str,
    ) -> Result<(), StrRet> {
        let executable_name = format!("{}{}", binary.name, env::consts::EXE_SUFFIX);

        let flags: String = match build_mode {
            "release" => format!("-Wall -Wextra -O3 {}", binary.flags.join(" ")),
            _ => format!("-g -ggdb -Wall -Wextra -Og {}", binary.flags.join(" ")),
        }
        .trim()
        .into();

        let build_path = format!(
            "{}/{}",
            project.get_directories().get_build_dir(),
            build_mode
        );
        if !Path::new(&build_path).exists() {
            create_dir_all(&build_path).expect("Could not create build output directory");
        }

        let compiler_command = project
            .get_compiler()
            .get_compiler_command(&binary.language);

        let mut command = Command::new(compiler_command);
        command.args(flags.split(' '));

        for library_directory in project.get_library_dirs() {
            command.arg(format!("-L{}", library_directory));
        }

        for library in project.get_libraries() {
            command.arg(format!("-l{}", library));
        }

        command
            .arg(binary.path.clone())
            .arg("-o")
            .arg(format!("{}/{}", build_path, executable_name));

        println!(
            "Compiling {} to {}...",
            binary.path.file_name().unwrap().to_str().unwrap(),
            executable_name
        );

        if let Err(e) = command.spawn().expect("Could not execute compiler").wait() {
            return Err(format!("Could not compile file: {}", e).into());
        }

        println!(
            "Compiled {} to {}",
            binary.path.file_name().unwrap().to_str().unwrap(),
            executable_name
        );

        Ok(())
    }

    pub fn build(args: &[String]) -> Result<(), StrRet> {
        let project = Self::get_toml(None, None)?;

        let mut build_mode = "debug";
        let mut is_verbose = false;
        let mut compiler_flags = String::from("");
        let mut bins = Vec::new();

        let executable_name =
            format!("{}{}", project.get_name(), env::consts::EXE_SUFFIX);

        let lock_file_path = "Ocean.lock";
        let lock_file = Path::new(&lock_file_path);

        let mut cache: Cache = if lock_file.exists() {
            let mut lock_file = File::open(lock_file).expect("Could not open Ocean.lock");
            let mut buffer = vec![];

            lock_file
                .read_to_end(&mut buffer)
                .expect("Could not read Ocean.lock");
            toml::from_str(
                String::from_utf8(buffer)
                    .expect("Could not read Ocean.lock content as valid UTF-8")
                    .as_str(),
            )
            .unwrap_or_else(|_| Cache::new(&project).unwrap())
        } else {
            Cache::new(&project)?
        };

        for directory in project.get_directories().get_all_dirs() {
            if !Path::new(directory).exists() {
                match create_dir_all(directory) {
                    Err(e) =>
                        println!("Could not create directory \"{}\": {}", directory, e),
                    _ => continue,
                }
            }
        }

        let file_extension = project.get_language().get_extension();
        let compiler = project
            .get_compiler()
            .get_compiler_command(project.get_language());

        for (index, arg) in args.iter().enumerate() {
            match arg.as_str() {
                "--help" => {
                    println!(
                        "
Usage: ocean build [OPTIONS] [-f [FLAGS]]

By default, this builds projects in debug mode.

Options:
    -d, --debug                 Builds the current project in debug mode (this is turned \
                         on by default).
    -r, --release               Builds the current project in release mode.
    --bin [all, <bin_name>]     Builds a single file as a single executable.
    -v, --verbose               Makes the compiler output verbose.
    -f, --flags                 Passes custom flags to the compiler.
            "
                    );
                    return Ok(());
                },
                "-r" | "--release" => build_mode = "release",
                "-d" | "--debug" => build_mode = "debug",
                "--bin" => bins.push(
                    if let Some(arg) = args.get(index as usize + 1) {
                        arg
                    } else {
                        return Err("Did not provide binary name or all as paramater \
                                    to --bin"
                            .into());
                    },
                ),
                "-v" | "--verbose" => is_verbose = true,
                "-f" | "--flags" =>
                    compiler_flags = args[index as usize..].to_vec().join(" "),
                _ => (),
            }
        }

        if !project.get_binaries().is_empty() && !bins.is_empty() {
            if *bins[0] == "all" {
                for binary in project.get_binaries().iter_mut() {
                    Self::build_file(&project, binary, build_mode)?
                }

                return Ok(());
            } else {
                for bin_name in bins.iter() {
                    for binary in project.get_binaries().iter_mut() {
                        if (*bin_name).clone() == binary.name {
                            Self::build_file(&project, binary, build_mode)?
                        } else {
                            continue;
                        }
                    }
                }
                return Ok(());
            }
        }

        let build_path = format!(
            "{}/{}",
            project.get_directories().get_build_dir(),
            build_mode
        );
        let object_path = format!(
            "{}/{}",
            project.get_directories().get_objects_dir(),
            build_mode
        );

        let mut compilable = vec![];

        let source_files =
            read_dir(project.get_directories().get_source_dir().to_string()).unwrap();
        'a: for file in source_files {
            let file_name = file.unwrap().path().clone();

            if Path::new(&file_name).is_dir() {
                continue;
            }

            for binary in project.get_binaries().into_iter() {
                if binary.path == file_name {
                    continue 'a;
                }
            }

            if Self::get_extension_from_filename(file_name.to_str().unwrap()).unwrap()
                == file_extension
            {
                compilable.push(file_name);
            }
        }

        if compilable.is_empty() {
            return Err("No compilable files found.".into());
        }

        if lock_file.exists() {
            let changed = cache.get_changed(&project)?;
            if !changed.is_empty() {
                compilable = compilable
                    .iter_mut()
                    .enumerate()
                    .filter(|(i, x)| *x == changed.get(*i - 1).unwrap_or(&PathBuf::new()))
                    .map(|x| x.1.clone())
                    .collect();
            } else if !Path::new(&format!("{}/{}", build_path, executable_name)).exists()
            {
                println!("Binary missing. Compiling anyway.");
            } else {
                println!("No compilation needed.");
                return Ok(());
            }
        }

        let lang_flags = if let Some(platform) = project.get_platform().clone() {
            let default = project
                .get_compiler()
                .get_compiler_flags(project.get_language())
                .clone();

            match env::consts::OS {
                "linux" =>
                    if let Some(linux) = platform.linux {
                        linux
                            .get_compiler()
                            .get_compiler_flags(project.get_language())
                            .to_owned()
                    } else {
                        default
                    },
                "osx" =>
                    if let Some(osx) = platform.osx {
                        osx.get_compiler()
                            .get_compiler_flags(project.get_language())
                            .to_owned()
                    } else {
                        default
                    },
                "windows" =>
                    if let Some(windows) = platform.windows {
                        windows
                            .get_compiler()
                            .get_compiler_flags(project.get_language())
                            .to_owned()
                    } else {
                        default
                    },
                _ => return Err("Unsupported operating system".into()),
            }
            .join(" ")
        } else {
            project
                .get_compiler()
                .get_compiler_flags(project.get_language())
                .join(" ")
        };

        let flag_arr = [lang_flags.trim(), compiler_flags.trim()].join(" ");
        let extra_flags = flag_arr.trim();

        let flags = match build_mode {
            "release" =>
                if !extra_flags.is_empty() {
                    format!("-Wall -Wextra -O3 {}", extra_flags)
                } else {
                    String::from("-Wall -Wextra -O3")
                },
            _ =>
                if !extra_flags.is_empty() {
                    format!("-g -ggdb -Wall -Wextra -Og {}", extra_flags)
                } else {
                    String::from("-g -ggdb -Wall -Wextra -Og")
                },
        };

        if !Path::new(&object_path).exists() {
            if let Err(e) = create_dir_all(object_path.clone()) {
                println!("Could not create directory \"{}\": {}", object_path, e);
            }
        }

        if !Path::new(&build_path).exists() {
            if let Err(e) = create_dir_all(build_path.clone()) {
                println!("Could not create directory \"{}\": {}", build_path, e);
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

            if let Err(e) = c
                .args(&flags.split(' ').collect::<Vec<&str>>())
                .arg("-c")
                .arg(file.to_str().unwrap())
                .spawn()
                .expect("Could not execute compiler")
                .wait()
            {
                return Err(
                    format!("Compiler command returned with error code: {}", e).into()
                );
            };

            if let Err(e) = rename(
                format!("{}.o", file.file_stem().unwrap().to_str().unwrap()),
                format!(
                    "{}/{}.o",
                    object_path,
                    file.file_stem().unwrap().to_str().unwrap()
                ),
            ) {
                return Err(format!(
                    "Cannot move object file: {}. Did the project compile properly?",
                    e
                )
                .into());
            }

            println!("Compiled {}.o", file.file_stem().unwrap().to_str().unwrap());
        }

        for file in read_dir(&object_path)
            .expect("Could not read objects directory")
            .into_iter()
        {
            if let Ok(file) = file {
                object_files.push(format!(
                    "{}/{}",
                    object_path,
                    file.file_name().to_str().unwrap()
                ));
            }
        }

        let mut c = Command::new(compiler);

        if is_verbose {
            c.arg("-v");
        }

        for obj in object_files {
            c.arg(obj);
        }

        c.arg("-o")
            .arg(format!("{}/{}", build_path, executable_name));

        for library_directory in project.get_library_dirs() {
            c.arg(format!("-L{}", library_directory));
        }

        for library in project.get_libraries() {
            c.arg(format!("-l{}", library));
        }

        if let Err(e) = c
            .spawn()
            .expect("Could not find compiler executable")
            .wait()
        {
            return Err(
                format!("Compiler command returned with error code: {}", e).into()
            );
        };

        let mut lock =
            File::create(lock_file).expect("Could not create/truncate Ocean.lock");
        cache.update_cache(&project)?;
        if !cache.get_files().is_empty() {
            lock.write_all(
                toml::to_string_pretty(&cache)
                    .expect("Could not convert into Cache")
                    .as_bytes(),
            )
            .expect("Could not write to Ocean.lock");
        } else {
            eprintln!("Could not find any files in source directory.");
            remove_file(lock_file).expect("Could not remove lock file");
        }

        Ok(())
    }

    pub fn run(args: &[String]) -> Result<(), StrRet> {
        let mut build_mode = "debug";
        let mut program_args = vec![];
        let mut bins = vec![];

        for (index, arg) in args.iter().enumerate() {
            match arg.as_str() {
                "--help" => {
                    println!(
                        "
Usage: ocean run [OPTIONS]

By default, this run projects in debug mode.

Options:
    -d, --debug                 Runs the current project in debug mode (this is turned \
                         on by default)
    -r, --release               Runs the current project in release mode
    --bin [all, <bin_name>]     Builds a single file as a single executable.
    -v, --verbose               Makes the compiler output verbose.
    -f, --flags                 Passes custom flags to the compiler.
            "
                    );
                    return Ok(());
                },
                "-r" | "--release" => build_mode = "release",
                "-d" | "--debug" => build_mode = "debug",
                "--bin" => bins.push(
                    if let Some(arg) = args.get(index as usize + 1) {
                        arg
                    } else {
                        return Err("Did not provide binary name or all as paramater \
                                    to --bin"
                            .into());
                    },
                ),
                "--" => program_args = args[index + 1..].to_vec(),
                _ => (), // -v and -f are handled by the build function
            }
        }

        Self::build(args)?;

        let project = Self::get_toml(None, None)?;

        let run = |name: String, program_args: &Vec<String>| -> Result<(), StrRet> {
            let executable_name = format!("{}{}", name, env::consts::EXE_SUFFIX);
            let executable_path = format!(
                "{}/{}/{}",
                project.get_directories().get_build_dir(),
                build_mode,
                executable_name
            );

            if Path::new(&executable_path).exists() {
                println!("\n[Running '{}']", executable_name);
                Command::new(format!("./{}", executable_path))
                    .args(program_args)
                    .spawn()
                    .expect("Could not start application")
                    .wait()
                    .expect("Application exited unexpectedly");

                Ok(())
            } else {
                Err(format!(
                    "Cannot find the \"{}\" executable. Did it compile properly?",
                    executable_name
                )
                .into())
            }
        };

        if !bins.is_empty() {
            let project_binaries = project.get_binaries();

            if bins[0] == "all" {
                for binary in project_binaries {
                    run(binary.name.clone(), &program_args)?;
                }
            } else {
                for bin_name in bins {
                    for binary in project_binaries
                        .iter()
                        .filter(|binary| binary.name == *bin_name)
                    {
                        run(binary.name.clone(), &program_args)?;
                    }
                }
            }

            return Ok(());
        }

        let executable_name =
            format!("{}{}", project.get_name(), env::consts::EXE_SUFFIX);
        run(executable_name, &program_args)?;

        Ok(())
    }

    pub fn new_project(args: &[String]) -> Result<(), String> {
        let mut project = Project::default();

        let mut do_ccls = false;
        let mut do_vscode = false;

        if !args.is_empty() {
            match args[0].as_str() {
                "--help" => {
                    println!(
                        "
Usage: ocean new [NAME] [OPTIONS]

This creates a new project with a generated Ocean.toml in a new directory with a \
                         specified NAME.
Options:
    -C                  Creates a new C project (default).
    -CXX                Creates a new C++ project.
    -b, --build-dir     Sets the build directory (default is \"./build\").
    -s, --source-dir    Sets the source directory (default is \"./src\").
    -o, --obj-dir       Sets the objects directory (default is \"./obj\").
    -c, --compiler      Sets the compiler for the current project (default is gcc for C \
                         and g++ for C++).
    --ccls              Outputs a .ccls file to be used with ccls. Allows a language \
                         server to be used with an editor like Vim, for example.
    --vscode            Outputs Visual Studio Code config files to make writing C/C++ \
                         easier.
            "
                    );
                    return Ok(());
                },
                _ => {
                    let name = args[0].to_string();
                    if !name.is_empty() {
                        project.set_name(name);
                    } else {
                        return Err("Did not specify project name".to_string());
                    }
                },
            }
        } else {
            return Err("Did not specify project name".to_string());
        }

        if Path::new(&format!("{}/Ocean.toml", project.get_name())).exists() {
            return Err("Cannot create a new project, Ocean.toml already exists in \
                        this directory."
                .to_string());
        }

        if Path::new(&format!("{}/", project.get_name())).exists()
            && read_dir(&format!("{}/", project.get_name()))
                .unwrap()
                .next()
                .is_none()
        {
            return Err("Cannot create a new project, directory is not empty".to_string());
        }

        for index in 0..args[1..].len() {
            let lang = *project.get_language();
            match args[index + 1].as_str() {
                "-C" => project.set_language(Language::C),
                "-CXX" => project.set_language(Language::CXX),
                "-b" | "--build-dir" => project.get_directories_mut().set_build_dir(
                    args.get(index + 2)
                        .expect("Did not specify a build directory")
                        .clone(),
                ),
                "-s" | "--source-dir" => project.get_directories_mut().set_source_dir(
                    args.get(index + 2)
                        .expect("Did not specify a source directory")
                        .clone(),
                ),
                "-o" | "--obj-dir" => project.get_directories_mut().set_objects_dir(
                    args.get(index + 2)
                        .expect("Did not specify an objects directory")
                        .clone(),
                ),
                "-c" | "--compiler" => project.get_compiler_mut().set_compiler_command(
                    lang,
                    args.get(index + 2)
                        .unwrap_or_else(|| {
                            panic!(("Did not specify custom {} compiler", lang))
                        })
                        .clone(),
                ),
                "--ccls" => do_ccls = true,
                "--vscode" => do_vscode = true,
                _ => (),
            }
        }

        let toml_content = toml::to_string_pretty(&project)
            .expect("Could not transform project data into Ocean.toml");
        let toml_content = Self::pretty_toml(toml_content);
        let code_content = match project.get_language() {
            Language::C =>
                "#include <stdio.h>

int main() {
    printf(\"Hello, world\\n\");
}
",
            Language::CXX =>
                "#include <iostream>

int main() {
    std::cout << \"Hello, world\" << std::endl;
}
",
        };

        create_dir_all(&format!("{}/src", project.get_name()))
            .expect("Could not create project and source directory");
        let mut file = File::create(&format!("{}/Ocean.toml", project.get_name()))
            .expect("Could not create Ocean.toml");
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
            .unwrap_or_else(|_| {
                panic!(
                    "Could not write to main.{}",
                    project.get_language().get_extension()
                )
            });

        #[cfg(feature = "git")]
        git::write_ignore(&project);

        if do_ccls {
            // TODO
            let _ = CCLS::new();
            let _ = File::create(&format!("{}/.ccls", project.get_name()))
                .expect("Could not create .ccls");
        }

        if do_vscode {
            let vscode = VsCode::new(&project);
            let config_dir = vscode.get_config_dir();
            let configs = vscode.get_config();
            create_dir_all(config_dir.clone())
                .expect("Could not create .vscode directory");

            let mut properties =
                File::create(format!("{}/c_cpp_properties.json", config_dir,))
                    .expect("Could not create c_cpp_properties.json");

            properties
                .write_all(
                    configs
                        .get("c_cpp_properties")
                        .expect("Could not find c_cpp_properties in hashmap.")
                        .as_bytes(),
                )
                .expect("Could not write to c_cpp_properties.json");

            let mut launch = File::create(format!("{}/launch.json", config_dir))
                .expect("Could not write lauch.json");

            launch
                .write_all(
                    configs
                        .get("launch")
                        .expect("Could not find launch in hashmap")
                        .as_bytes(),
                )
                .expect("Could not write to tasks.json");

            let mut tasks = File::create(format!("{}/tasks.json", config_dir))
                .expect("Could not write tasks.json");

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

    pub fn clean() -> Result<(), String> {
        let project = Self::get_toml(None, None)?;

        for directory in project.get_directories().get_all_dirs() {
            if directory == project.get_directories().get_source_dir() {
                continue;
            }

            remove_file("Ocean.lock").unwrap_or(());
            remove_dir_all(directory).unwrap_or(());
        }

        Ok(())
    }

    pub fn set_data(args: &[String]) -> Result<(), String> {
        let mut project = Self::get_toml(None, None)?;
        let mut do_clean = true;

        let help = "
Usage: ocean set [KEY]

This sets values inside the Ocean project file to a value specified by the user. When \
                    provided with a platform name, it allows the user to set specific \
                    keys for a specified platform.

Option:
    build_dir [DIRECTORY]                               Sets the build directory for the \
                    project.
    c++_compiler [COMPILER], cxx_compiler [COMPILER]    Set the compiler being used for \
                    the C++ project.
    c_compiler [COMPILER]                               Sets the compiler being used for \
                    the C project.
    compiler [COMPILER], current_compiler [COMPILER]    Sets the current compiler being \
                    used for the project.
    flags [FLAGS]                                       Sets the flags of the current \
                    compiler, split by commas.
    lang [LANG], language [LANG]                        Set the current language of the \
                    project.
    lib_dirs [DIRS], library_directories [DIRS]         Sets the library directories \
                    that would be searched by the linker, split by commas.
    libs [LIBS], libraries [LIBS]                       Sets the libraries being \
                    compiled with the project, split by commas.
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

        if !args[1..].is_empty() {
            match (args[0].as_str(), &args[1]) {
                ("name", n) => {
                    project.set_name(n.clone());
                    do_clean = false;
                },
                (c, lang) if c == "lang" || c == "language" =>
                    match lang.to_lowercase().as_str() {
                        "c++" | "cxx" => project.set_language(Language::CXX),
                        "c" => project.set_language(Language::C),
                        _ => return Err("Invalid language.".to_string()),
                    },
                (c, libs) if c == "libs" || c == "libraries" =>
                    for lib in libs.split(',') {
                        project.add_library(lib.to_string());
                    },
                (c, dirs) if c == "lib_dirs" || c == "library_directories" =>
                    for dir in dirs.split(',') {
                        project.add_library_directories(dir.to_string());
                    },
                ("c_compiler", compiler) =>
                    project.set_compiler(Language::C, compiler.clone()),
                (c, compiler) if c == "c++_compiler" || c == "cxx_compiler" =>
                    project.set_compiler(Language::CXX, compiler.clone()),
                (c, compiler) if c == "compiler" || c == "current_compiler" =>
                    project.set_current_compiler(compiler.clone()),
                ("object_dir", dir) =>
                    project.get_directories_mut().set_objects_dir(dir.clone()),
                ("source_dir", dir) =>
                    project.get_directories_mut().set_source_dir(dir.clone()),
                ("build_dir", dir) =>
                    project.get_directories_mut().set_build_dir(dir.clone()),
                ("flags", flags) => {
                    let flags_vec = {
                        let mut v = vec![];
                        let flags_vec_str: Vec<&str> = flags.split(',').collect();
                        flags_vec_str.iter().for_each(|f| v.push(f.to_string()));
                        v
                    };

                    let lang = *project.get_language();
                    project
                        .get_compiler_mut()
                        .set_compiler_flags(lang, flags_vec);
                },
                _ => return Err("Incorrect data key.".to_string()),
            }
        } else {
            return Err("No data provided to set key with".to_string());
        }

        if do_clean {
            Self::clean()?
        }

        let mut file = File::create("./Ocean.toml").expect("Couldn't open Ocean.toml");
        let toml_content = toml::to_string_pretty(&project)
            .expect("Could not transform project data into Ocean.toml");
        let toml_content = Self::pretty_toml(toml_content);
        file.write_all(toml_content.as_bytes())
            .expect("Could not write to Ocean.toml");

        Ok(())
    }

    pub fn get_data(args: &[String]) -> Result<(), String> {
        let project = Self::get_toml(None, None)?;

        let help = "
Usage: ocean get [KEY]

This gets the current values inside the Ocean project file related to a data key entered \
                    by the user. When given a platform name, it will get specific keys \
                    for a specified platform.

Option:
    bins, binaries                  Prints the names of the individual binaries for the \
                    current project.
    build_dir                       Prints the build directory for the current project.
    c++_compiler, cxx_compiler      Prints the compiler being used for the C++ project.
    c_compiler                      Prints the compiler being used for the C project.
    compiler, current_compiler      Prints the current compiler being used for the \
                    project.
    flags                           Prints the flags of the current compiler.
    lang, language                  Prints the current language of the project.
    lib_dirs, library_directories   Prints the library directories that would be \
                    searched by the linker.
    libs, libraries                 Prints the libraries being compiled with the \
                    project.  
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
            "lib_dirs" | "library_directories" =>
                println!("{:#?}", project.get_directories().get_all_dirs()),
            "compiler" | "current_compiler" => println!(
                "{}",
                project
                    .get_compiler()
                    .get_compiler_command(project.get_language())
                    .clone()
            ),
            "c_compiler" => println!(
                "{}",
                project
                    .get_compiler()
                    .get_compiler_command(&Language::C)
                    .clone()
            ),
            "c++_compiler" | "cxx_compiler" => println!(
                "{}",
                project
                    .get_compiler()
                    .get_compiler_command(&Language::CXX)
                    .clone()
            ),
            "build_dir" => println!("{}", project.get_directories().get_build_dir()),
            "source_dir" => println!("{}", project.get_directories().get_source_dir()),
            "object_dir" => println!("{}", project.get_directories().get_objects_dir()),
            "flags" => println!(
                "{:#?}",
                project
                    .get_compiler()
                    .get_compiler_flags(project.get_language())
            ),
            "bins" | "binaries" => println!("{:#?}", project.get_binaries()),
            _ => eprintln!(
                "Cannot find data key. Use --help to get help for this command."
            ),
        };
        Ok(())
    }

    pub fn set_data_platform(
        args: &[String],
        platform_name: String,
    ) -> Result<(), String> {
        let mut project = Self::get_toml(None, None)?;

        let mut platform = if let Some(plats) = project.get_platform_mut() {
            plats.clone()
        } else {
            Platforms::default()
        };

        let mut plat_ops = PlatformOptions {
            platform_name: platform_name.clone(),
            ..PlatformOptions::default()
        };

        let help = "
Usage: ocean set [PLATFORM] [KEY]

This set values inside the Ocean project file to a value for a specified platform.

Option:
    c++_compiler [COMPILER], cxx_compiler [COMPILER]    Set the compiler being used for \
                    the C++ project.
    c_compiler [COMPILER]                               Sets the compiler being used for \
                    the C project.
    compiler [COMPILER], current_compiler [COMPILER]    Sets the current compiler being \
                    used for the project.
    flags [FLAGS]                                       Sets the flags of the current \
                    compiler, split by commas.
    lib_dirs [DIRS], library_directories [DIRS]         Sets the library directories \
                    that would be searched by the linker, split by commas.
    libs [LIBS], libraries [LIBS]                       Sets the libraries being \
                    compiled with the project, split by commas.
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

        if !args[1..].is_empty() {
            match (args[0].as_str(), &args[1]) {
                (c, libs) if c == "libs" || c == "libraries" =>
                    for lib in libs.split(',') {
                        plat_ops.add_library(lib.to_string());
                    },
                (c, dirs) if c == "lib_dirs" || c == "library_directories" =>
                    for dir in dirs.split(',') {
                        plat_ops.add_library_directories(dir.to_string());
                    },
                ("c_compiler", compiler) =>
                    project.set_compiler(Language::C, compiler.clone()),
                (c, compiler) if c == "c++_compiler" || c == "cxx_compiler" =>
                    plat_ops.set_compiler(Language::CXX, compiler.clone()),
                (c, compiler) if c == "compiler" || c == "current_compiler" =>
                    plat_ops.set_compiler(*project.get_language(), compiler.clone()),
                ("flags", flags) => {
                    let flags_vec = {
                        let mut v = vec![];
                        let flags_vec_str: Vec<&str> = flags.split(',').collect();
                        flags_vec_str.iter().for_each(|f| v.push(f.to_string()));
                        v
                    };

                    let lang = *project.get_language();
                    plat_ops
                        .get_compiler_mut()
                        .set_compiler_flags(lang, flags_vec);
                },
                _ => return Err("Incorrect data key.".to_string()),
            }
        } else {
            return Err("No data provided to set key with".to_string());
        }

        *project.get_platform_mut() = Some({
            match platform_name.as_str() {
                "linux" => platform.linux = Some(plat_ops),
                "osx" => platform.osx = Some(plat_ops),
                "windows" => platform.windows = Some(plat_ops),
                _ => return Err("Invalid platform".to_string()),
            };

            platform
        });

        Self::clean()?;

        let mut file = File::create("./Ocean.toml").expect("Couldn't open Ocean.toml");
        let toml_content = toml::to_string_pretty(&project)
            .expect("Could not transform project data into Ocean.toml");
        let toml_content = Self::pretty_toml(toml_content);
        file.write_all(toml_content.as_bytes())
            .expect("Could not write to Ocean.toml");

        Ok(())
    }

    pub fn get_data_platform(
        args: &[String],
        platform_name: String,
    ) -> Result<(), String> {
        let project = Self::get_toml(None, None)?;

        let plat_ops = if let Some(plats) = project.get_platform() {
            match platform_name.as_str() {
                "linux" =>
                    if let Some(linux) = plats.linux.clone() {
                        linux
                    } else {
                        return Err("Could not find any keys for Linux".to_string());
                    },
                "osx" =>
                    if let Some(osx) = plats.osx.clone() {
                        osx
                    } else {
                        return Err("Could not find any keys for OSX".to_string());
                    },
                "windows" =>
                    if let Some(windows) = plats.windows.clone() {
                        windows
                    } else {
                        return Err("Could not find any keys for Windows".to_string());
                    },
                _ => return Err("Could not find the specified platform".to_string()),
            }
        } else {
            return Err("Could not find platform".to_string());
        };

        let help = "
Usage: ocean get [PLATFORM] [KEY]

This get values inside the Ocean project file to a value for a specified platform.

Option:
    c++_compiler , cxx_compiler    Get the compiler being used for the C++ project.
    c_compiler                     Get the compiler being used for the C project.
    compiler, current_compiler     Get the current compiler being used for the project.
    flags                          Get the flags of the current compiler, split by \
                    commas.
    lib_dirs, library_directories  Get the library directories that would be searched by \
                    the linker, split by commas.
    libs, libraries                Get the libraries being compiled with the project, \
                    split by commas.
    ";

        let data = args[0].as_str();

        match data {
            "--help" => println!("{}", help),
            "libs" | "libraries" => println!("{:#?}", plat_ops.get_libraries()),
            "lib_dirs" | "library_directories" =>
                println!("{:#?}", plat_ops.get_library_dirs()),
            "compiler" | "current_compiler" => println!(
                "{}",
                plat_ops
                    .get_compiler()
                    .get_compiler_command(project.get_language())
            ),
            "c_compiler" => println!(
                "{}",
                plat_ops
                    .get_compiler()
                    .get_compiler_command(&Language::C)
                    .clone()
            ),
            "c++_compiler" | "cxx_compiler" => println!(
                "{}",
                plat_ops
                    .get_compiler()
                    .get_compiler_command(&Language::CXX)
                    .clone()
            ),
            "flags" => println!(
                "{:#?}",
                plat_ops
                    .get_compiler()
                    .get_compiler_flags(project.get_language())
            ),
            _ => eprintln!(
                "Cannot find data key. Use --help to get help for this command."
            ),
        };

        Ok(())
    }

    // TODO(#5) Submodule command
}
