use crate::{compiler::*, language::*, platform::*};
use std::{collections::hash_map::Values, collections::HashMap, env, path::PathBuf};

#[derive(Deserialize, Serialize)]
pub struct DirectoryHashMap(HashMap<String, String>);

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Binary {
    pub name: String,
    pub path: PathBuf,
    pub language: Language,
    pub flags: Vec<String>,
}

impl DirectoryHashMap {
    pub fn new() -> Self {
        Self {
            0: {
                let mut hm = HashMap::new();
                hm.insert("build_dir".to_string(), "./build".to_string());
                hm.insert("object_dir".to_string(), "./obj".to_string());
                hm.insert("source_dir".to_string(), "./src".to_string());

                hm
            },
        }
    }

    pub fn get_all_dirs(&self) -> Values<'_, String, String> { self.0.values() }
    pub fn get_build_dir(&self) -> &String { &self.0["build_dir"] }
    pub fn get_objects_dir(&self) -> &String { &self.0["object_dir"] }
    pub fn get_source_dir(&self) -> &String { &self.0["source_dir"] }

    pub fn set_build_dir(&mut self, dir: String) {
        let build_dir = self.0.get_mut("build_dir").expect("Could not find build_dir key.");
        *build_dir = dir;
    }

    pub fn set_source_dir(&mut self, dir: String) {
        let source_dir = self.0.get_mut("source_dir").expect("Could not find source_dir key.");
        *source_dir = dir;
    }

    pub fn set_objects_dir(&mut self, dir: String) {
        let object_dir = self.0.get_mut("object_dir").expect("Could not find object_dir key.");
        *object_dir = dir;
    }
}

#[derive(Deserialize, Serialize)]
struct Inner {
    name: String,
    language: Language,
    libraries: Vec<String>,
    library_directories: Vec<String>,
}

impl Default for Inner {
    fn default() -> Self {
        Self {
            name: "Ocean Project".to_string(),
            language: Language::C,
            libraries: Vec::default(),
            library_directories: Vec::default(),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Project {
    project: Inner,
    directories: DirectoryHashMap,
    compiler: Compiler,
    platforms: Option<Platforms>,
    bins: Option<Vec<Binary>>,
}

impl Project {
    pub fn get_compiler(&self) -> &Compiler { &self.compiler }
    pub fn get_compiler_mut(&mut self) -> &mut Compiler { &mut self.compiler }
    pub fn get_directories(&self) -> &DirectoryHashMap { &self.directories }
    pub fn get_directories_mut(&mut self) -> &mut DirectoryHashMap { &mut self.directories }
    pub fn get_language(&self) -> &Language { &self.project.language }

    pub fn get_libraries(&self) -> &Vec<String> {
        if let Some(platform) = &self.platforms {
            if let Some(pl) = match env::consts::OS {
                "linux" => &platform.linux,
                "osx" => &platform.osx,
                "windows" => &platform.windows,
                _ => &None,
            } {
                return &pl.libraries;
            }
        }

        &self.project.libraries
    }

    pub fn get_library_dirs(&self) -> &Vec<String> {
        if let Some(platform) = &self.platforms {
            if let Some(pl) = match env::consts::OS {
                "linux" => &platform.linux,
                "osx" => &platform.osx,
                "windows" => &platform.windows,
                _ => &None,
            } {
                return &pl.libraries;
            }
        }

        &self.project.library_directories
    }

    pub fn get_name(&self) -> &String { &self.project.name }
    pub fn get_platform(&self) -> &Option<Platforms> { &self.platforms }
    pub fn get_platform_mut(&mut self) -> &mut Option<Platforms> { &mut self.platforms }
    pub fn set_language(&mut self, lang: Language) { self.project.language = lang; }
    pub fn set_name(&mut self, name: String) { self.project.name = name; }

    pub fn add_library(&mut self, lib_path: String) {
        println!("Added the '{}' library", lib_path);
        self.project.libraries.push(lib_path);
    }

    pub fn add_library_directories(&mut self, lib_dir: String) {
        println!("Added '{}' to the library directories", lib_dir);
        self.project.library_directories.push(lib_dir);
    }

    pub fn set_compiler(&mut self, language: Language, compiler_command: String) {
        println!(
            "Set compiler command for {} to '{}'",
            language.as_string(),
            compiler_command
        );
        self.get_compiler_mut().set_compiler_command(language, compiler_command);
    }

    pub fn set_current_compiler(&mut self, compiler_command: String) {
        println!(
            "Set compiler command for {} to '{}'",
            self.project.language.as_string(),
            compiler_command
        );

        let language = self.project.language;

        self.get_compiler_mut().set_compiler_command(language, compiler_command);
    }

    pub fn get_binaries(&self) -> Vec<Binary> {
        if let Some(bins) = self.bins.clone() {
            return bins;
        }

        Vec::new()
    }
}

impl Default for Project {
    fn default() -> Self {
        Self {
            project: Inner::default(),
            directories: DirectoryHashMap::new(),
            compiler: Compiler::default(),
            platforms: None,
            bins: None,
        }
    }
}
