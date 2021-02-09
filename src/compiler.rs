use crate::language::*;

#[derive(Deserialize, Serialize, Clone)]
pub struct CompilerOptions {
    command: String,
    flags: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Compiler {
    c: CompilerOptions,
    cxx: CompilerOptions,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            cxx: CompilerOptions {
                command: String::from("g++"),
                flags: vec![],
            },

            c: CompilerOptions {
                command: String::from("gcc"),
                flags: vec![],
            },
        }
    }

    pub fn get_compiler_command(&self, lang: &Language) -> &String {
        match lang {
            Language::C => &self.c.command,
            Language::CXX => &self.cxx.command,
        }
    }

    pub fn set_compiler_command(&mut self, lang: Language, compiler_command: String) {
        match lang {
            Language::C => self.c.command = compiler_command,
            Language::CXX => self.cxx.command = compiler_command,
        }
    }

    pub fn get_compiler_flags(&self, lang: &Language) -> &Vec<String> {
        match lang {
            Language::C => &self.c.flags,
            Language::CXX => &self.cxx.flags,
        }
    }
    pub fn set_compiler_flags(&mut self, lang: Language, flags: Vec<String>) {
        match lang {
            Language::C => self.c.flags = flags,
            Language::CXX => self.cxx.flags = flags,
        }
    }
}

impl Default for Compiler {
    fn default() -> Self { Self::new() }
}

impl From<[CompilerOptions; 2]> for Compiler {
    fn from(co: [CompilerOptions; 2]) -> Self {
        Self {
            cxx: co[0].clone(),
            c: co[1].clone(),
        }
    }
}
