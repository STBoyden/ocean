use crate::{compiler::*, language::*};
use std::env;

#[derive(Deserialize, Serialize, Clone)]
pub struct PlatformOptions {
    pub platform_name: String,
    pub libraries: Vec<String>,
    pub library_directories: Vec<String>,
    pub compiler: Compiler,
}

impl PlatformOptions {
    pub fn get_compiler(&self) -> &Compiler { &self.compiler }
    pub fn get_compiler_mut(&mut self) -> &mut Compiler { &mut self.compiler }
    pub fn get_libraries(&self) -> &Vec<String> { &self.libraries }
    pub fn get_library_dirs(&self) -> &Vec<String> { &self.library_directories }

    pub fn add_library(&mut self, lib_path: String) {
        println!("Added the '{}' library for {}", lib_path, self.platform_name);
        self.libraries.push(lib_path);
    }

    pub fn add_library_directories(&mut self, lib_dir: String) {
        println!(
            "Added '{}' to the library directories for {}",
            lib_dir, self.platform_name
        );
        self.library_directories.push(lib_dir);
    }

    pub fn set_compiler(&mut self, language: Language, compiler_command: String) {
        println!(
            "Set compiler command for {} to '{}' for {}",
            language.as_string(),
            compiler_command,
            self.platform_name
        );
        self.get_compiler_mut().set_compiler_command(language, compiler_command);
    }
}

impl Default for PlatformOptions {
    fn default() -> Self {
        Self {
            platform_name: {
                let mut n = String::from(env::consts::OS);

                {
                    let (first_char, _) = n.split_at_mut(1);
                    first_char.make_ascii_uppercase();
                }

                n
            },
            compiler: Compiler::default(),
            libraries: Vec::default(),
            library_directories: Vec::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Platforms {
    pub linux: Option<PlatformOptions>,
    pub bsd: Option<PlatformOptions>,
    pub osx: Option<PlatformOptions>,
    pub windows: Option<PlatformOptions>,
}

impl Default for Platforms {
    fn default() -> Self {
        Self {
            linux: None,
            bsd: None,
            osx: None,
            windows: None,
        }
    }
}
