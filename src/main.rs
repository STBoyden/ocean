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

fn main() {

}
