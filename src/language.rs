use serde_derive::*;
use std::fmt;

#[derive(Deserialize, Serialize)]
pub enum Language {
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

impl Language {
    pub fn get_compiler(&self) -> String {
        match self {
            Language::C => "gcc",
            Language::CXX => "g++",
        }
        .to_string()
    }
    pub fn get_extension(&self) -> String {
        match self {
            Language::C => "c",
            Language::CXX => "cpp",
        }
        .to_string()
    }
}
