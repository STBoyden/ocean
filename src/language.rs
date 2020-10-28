use serde_derive::*;

#[derive(Deserialize, Serialize, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Language {
    C,
    CXX,
}

impl Language {
    pub fn as_string(&self) -> String {
        match self {
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
