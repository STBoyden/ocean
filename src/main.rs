use serde_derive::*;
use std::collections::HashMap;
use std::fmt;

#[derive(Deserialize, Serialize)]
enum Language {
    C,
    CXX,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let lang = match self {
            Language::C => "C",
            Language::CXX => "C++",
        };

        write!(f, "{}", lang)
    }
}

#[derive(Deserialize, Serialize)]
struct Project {
    name: String,
    language: Language,
    directories: HashMap<String, String>,
}

fn help(argument: Option<&String>) {
    if argument.is_some() {
        print!("Command \"{}\" not found.\n", argument.unwrap());
    }
    println!(
        "
Usage: ocean [OPTION]

Create and manage C and C++ projects.

    build           Builds the current project
    run             Runs the current project, builds if no build is present
    new             Creates a new C/C++ project in a new directory
    init            Creates a new C/C++ project in the current directory as long as it is empty
    clean           Cleans the current project's build artifacts
        "
    );
}
fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        args = args[1..].to_vec();
    } else {
        println!("No arguments specified");
        help(None);
        return;
    }

    let project = {
        let mut contents = String::from("");

        match File::open("Ocean_Project.toml") {
            Ok(mut f) => match f.read_to_string(&mut contents) {
                Err(_) => {
                    println!("Could not read file");
                    return;
                }
                _ => {}
            },
            _ => {}
        }

        toml::from_str(contents.as_str()).unwrap_or(Project {
            name: String::from("Ocean Project"),
            language: Language::C,
            directories: {
                let mut hm = HashMap::new();

                hm.insert("build_dir".to_string(), "./build".to_string());
                hm.insert("source_dir".to_string(), "./src".to_string());
                hm.insert("object_dir".to_string(), "./obj".to_string());

                hm
            },
        })
    };

    match args[0].as_str() {
        "help" => help(None),
        _ => help(Some(&args[0])),
    }
}
