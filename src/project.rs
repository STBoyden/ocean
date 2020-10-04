use crate::language::*;
use serde_derive::*;
use std::{collections::hash_map::Values, collections::HashMap};

#[derive(Deserialize, Serialize)]
pub struct DirectoryHashMap(HashMap<String, String>);

#[derive(Deserialize, Serialize)]
pub struct Project {
    name: String,
    language: Language,
    libraries: Vec<String>,
    library_directories: Vec<String>,
    directories: DirectoryHashMap,
    compiler: Compiler,
}

impl DirectoryHashMap {
    pub fn new() -> Self {
        Self {
            0: {
                let mut hm = HashMap::new();
                hm.insert("build_dir".to_string(), "./build".to_string());
                hm.insert("source_dir".to_string(), "./src".to_string());
                hm.insert("object_dir".to_string(), "./obj".to_string());

                hm
            },
        }
    }

    pub fn get_all_dirs(&self) -> Values<'_, String, String> {
        self.0.values()
    }

    pub fn get_build_dir(&self) -> &String {
        &self.0["build_dir"]
    }

    pub fn get_source_dir(&self) -> &String {
        &self.0["source_dir"]
    }

    pub fn get_objects_dir(&self) -> &String {
        &self.0["object_dir"]
    }

    pub fn set_build_dir(&mut self, dir: String) {
        self.0
            .remove_entry("build_dir")
            .expect("Could not find \"build_dir\" in keys");

        self.0.insert("build_dir".to_string(), dir);
    }

    pub fn set_source_dir(&mut self, dir: String) {
        self.0
            .remove_entry("source_dir")
            .expect("Could not find \"source_dir\" in keys");

        self.0.insert("source_dir".to_string(), dir);
    }

    pub fn set_objects_dir(&mut self, dir: String) {
        self.0
            .remove_entry("object_dir")
            .expect("Could not find \"object_dir\" in keys");

        self.0.insert("object_dir".to_string(), dir);
    }
}

impl Project {
    pub fn get_compiler(&self) -> &Compiler {
        &self.compiler
    }

    pub fn get_compiler_mut(&mut self) -> &mut Compiler {
        &mut self.compiler
    }

    pub fn get_directories(&self) -> &DirectoryHashMap {
        &self.directories
    }

    pub fn get_directories_mut(&mut self) -> &mut DirectoryHashMap {
        &mut self.directories
    }

    pub fn get_language(&self) -> &Language {
        &self.language
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_libraries(&self) -> &Vec<String> {
        &self.libraries
    }

    pub fn get_library_dirs(&self) -> &Vec<String> {
        &self.library_directories
    }

    pub fn set_language(&mut self, lang: Language) {
        self.language = lang;
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: "Ocean Project".to_string(),
            language: Language::C,
            libraries: Vec::new(),
            library_directories: Vec::new(),
            directories: DirectoryHashMap::new(),
            compiler: Compiler::new(),
        }
    }
}
