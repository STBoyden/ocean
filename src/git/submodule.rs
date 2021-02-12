use std::{
    fs::create_dir,
    path::Path,
    process::{self, Stdio},
};
use url::Url;

fn default_path() -> String { "third_party".into() }
fn default_branch() -> String { "master".into() }

#[derive(Serialize, Deserialize)]
pub struct Submodules {
    #[serde(default = "default_path")]
    pub path: String,
    #[serde(default, rename(serialize = "module", deserialize = "module"))]
    pub submodules: Vec<Submodule>,
}

impl Submodules {
    // TODO(#5) Submodules update
    // TODO(#5) Submodules remove
    // TODO(#5) Submodules build
    pub fn add_submodule<S: Into<String>>(
        &mut self,
        origin: S,
        branch: Option<String>,
        directory_name: Option<String>,
    ) {
        let branch = branch.unwrap_or_else(default_branch);
        let origin: String = origin.into();

        if !Path::new(&self.path).exists() {
            create_dir(&self.path).expect(&format!(
                "Could not create '{}' submodule directory",
                self.path
            ));
        }

        self.submodules
            .push(Submodule::new(origin.clone(), branch.clone()));

        let mut command = process::Command::new("git");
        command.args(&["submodule", "add", origin.as_str(), "-b", branch.as_str()]);

        if let Some(directory_name) = directory_name {
            command.arg(format!("{}/{}", self.path, directory_name));
        } else {
            let directory_name = &Url::parse(&origin)
                .expect(&format!("Could not parse '{}' as URL", origin))
                .path_segments()
                .expect(&format!(
                    "Could not get path segments of '{}'. Are you sure you entered a \
                     Git repository?",
                    origin
                ))
                .map(|x| x.to_owned())
                .collect::<Vec<String>>()[0];

            command.arg(format!("{}/{}", default_path(), directory_name));
        }

        command
            .stderr(Stdio::null())
            .spawn()
            .expect("Could not spawn Git")
            .wait()
            .expect("Git did not run");
    }
}

impl Default for Submodules {
    fn default() -> Self {
        Self {
            path: default_path(),
            submodules: vec![],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Submodule {
    origin: String,
    #[serde(default = "default_branch")]
    branch: String,
}

impl Submodule {
    pub fn new<S: Into<String>>(origin: S, branch: S) -> Self {
        Self {
            origin: origin.into(),
            branch: branch.into(),
        }
    }

    // TODO(#5) Submodule update
    // TODO(#5) Submodule detect build system
    // TODO(#5) Submodule build
}
