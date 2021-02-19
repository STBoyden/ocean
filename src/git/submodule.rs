use std::{
    collections::HashMap,
    fs::create_dir,
    path::Path,
    process::{Command, Stdio},
};
use url::Url;

fn default_path() -> String { "third_party".into() }
fn default_branch() -> String { "master".into() }

#[derive(Serialize, Deserialize)]
pub struct Submodules {
    #[serde(default = "default_path")]
    pub path: String,
    #[serde(default, rename(serialize = "module", deserialize = "module"))]
    pub submodules: HashMap<String, Submodule>,
}

impl Submodules {
    // TODO(#5) Submodules update
    pub fn _update_all(&self) { unimplemented!() }

    pub fn remove_submodule<S: Into<String>>(&mut self, directory_name: S) {
        // TODO(#5) Submodules remove: spawn git command to remove submodules from
        // .gitsubmodules and third_party/{directory_name}.
        self.submodules.remove(&directory_name.into());
    }

    // TODO(#5) Submodules build
    pub fn _build_all(&self) { unimplemented!() }

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

        let mut command = Command::new("git");
        command.args(&["submodule", "add", origin.as_str(), "-b", branch.as_str()]);

        let submodule = Submodule::new(origin, branch);

        if let Some(directory_name) = directory_name {
            command.arg(format!("{}/{}", self.path, directory_name));
        } else {
            command.arg(format!("{}/{}", self.path, submodule.directory_name));
        }

        self.submodules
            .insert(submodule.directory_name.clone(), submodule);

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
            submodules: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Submodule {
    origin: String,
    #[serde(default = "default_branch")]
    branch: String,
    directory_name: String,
}

impl Submodule {
    pub fn new<S: Into<String>>(origin: S, branch: S) -> Self {
        let origin = origin.into();
        Self {
            origin: origin.clone(),
            branch: branch.into(),
            directory_name: {
                let s = Url::parse(&origin)
                    .expect(&format!("Could not parse '{}' as URL", origin));

                s.path_segments()
                    .expect(&format!(
                        "Could not get path segments of '{}'. Are you sure you entered \
                         a Git repository?",
                        origin
                    ))
                    .map(|x| x.to_owned())
                    .collect::<Vec<String>>()[0]
                    .clone()
            },
        }
    }

    // TODO(#5) Submodule detect build system
    // TODO(#5) Submodule build
}
