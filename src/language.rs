use serde_derive::*;
use std::{collections::HashMap, fmt};

#[derive(Deserialize, Serialize)]
pub struct Compiler(HashMap<String, String>);

impl Compiler {
    pub fn new() -> Self {
        Self {
            0: {
                let mut hm = HashMap::new();

                hm.insert(Language::C.as_string(), "gcc".to_string());
                hm.insert(Language::CXX.as_string(), "g++".to_string());

                hm
            },
        }
    }

    pub fn get_compiler_command(&self, lang: &Language) -> &String {
        self.0.get(&lang.as_string()).unwrap()
    }

    pub fn set_compiler_command<T: Into<String>>(&mut self, lang: Language, compiler: T) {
        self.0
            .remove(&lang.as_string())
            .expect("Could not find language in HashMap");

        self.0.insert(lang.as_string(), compiler.into());
    }
}

#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Language {
    C,
    CXX,
}

impl Language {
    pub fn as_string(&self) -> String {
        use Language::*;
        match self {
            C => "C".to_string(),
            CXX => "CXX".to_string(),
        }
    }

    pub fn get_extension(&self) -> String {
        match self {
            Language::C => "c",
            Language::CXX => "cpp",
        }
        .to_string()
    }
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
