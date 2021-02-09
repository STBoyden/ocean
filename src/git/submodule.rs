fn default_path() -> String { "third_party".into() }

#[derive(Serialize, Deserialize)]
pub struct Submodules {
    #[serde(default = "default_path")]
    pub path: String,
    #[serde(default, rename(serialize = "module", deserialize = "module"))]
    pub submodules: Vec<Submodule>,
}

impl Default for Submodules {
    fn default() -> Self {
        Self {
            path: default_path(),
            submodules: vec![],
        }
    }
}

fn default_branch() -> String { "master".into() }

#[derive(Serialize, Deserialize)]
pub struct Submodule {
    pub origin: String,
    #[serde(default = "default_branch")]
    pub branch: String,
    #[serde(default)]
    pub tag: Option<String>,
}

impl Submodule {
    pub fn new<S: Into<String>>(origin: S, branch: S, tag: Option<String>) -> Self {
        Self {
            origin: origin.into(),
            branch: branch.into(),
            tag,
        }
    }

    // TODO(#4) Submodule add
    // TODO(#4) Submodule update
    // TODO(#4) Submodule detect build system
    // TODO(#4) Submodule build
}
