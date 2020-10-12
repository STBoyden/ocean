use crate::Project;
use std::{collections::HashMap, env, path::Path};

pub trait Editor<T> {
    fn get_compiler_path(project: &Project) -> Result<String, String> {
        let mut ret_path = String::new();

        match env::var_os("PATH") {
            Some(path_var) => {
                for path in env::split_paths(&path_var) {
                    if Path::new(
                        format!(
                            "{}/{}.exe",
                            path.to_str().unwrap(),
                            project.get_compiler().get_compiler_command(project.get_language())
                        )
                        .as_str(),
                    )
                    .is_file()
                        && env::consts::OS == "windows"
                    {
                        ret_path = path.to_str().unwrap().to_string();
                    } else if Path::new(
                        format!(
                            "{}/{}",
                            path.to_str().unwrap(),
                            project.get_compiler().get_compiler_command(project.get_language())
                        )
                        .as_str(),
                    )
                    .is_file()
                        && cfg!(unix)
                    {
                        ret_path = path.to_str().unwrap().to_string();
                    }
                }
            }
            None => return Err("Cannot find PATH environment variable".to_string()),
        };

        if ret_path == "" {
            return Err("Cannot find compiler in PATH. Did you add the compiler's directory to the PATH?".to_string());
        }

        Ok(ret_path)
    }
    fn get_config_dir(&self) -> String;
    fn get_config(&self) -> T;
}

pub struct CCLS {
    config_dir: String,
    config: String,
}

impl CCLS {
    pub fn new() -> Self {
        Self {
            config_dir: String::new(),
            config: String::new(),
        }
    }
}

impl Editor<String> for CCLS {
    fn get_config_dir(&self) -> String { self.config_dir.clone() }
    fn get_config(&self) -> String { self.config.clone() }
}

type VscConfigs = HashMap<String, String>;
pub struct VsCode {
    config_dir: String,
    configs: VscConfigs,
}

impl Editor<VscConfigs> for VsCode {
    fn get_config_dir(&self) -> String { self.config_dir.clone() }
    fn get_config(&self) -> VscConfigs { self.configs.clone() }
}

impl VsCode {
    pub fn new(project: &Project) -> Self {
        Self {
            config_dir: format!("{}/.vscode", project.get_name()),
            configs: VscConfigs::new(),
        }
        .init(project)
    }

    fn init(mut self, project: &Project) -> Self {
        let mut command = project
            .get_compiler()
            .get_compiler_command(project.get_language())
            .clone();
        if command == "g++" {
            command = "gcc".to_string();
        }

        self.configs.insert(
            "c_cpp_properties".to_string(),
            format!(
                "{{
    \"configurations\": [
        {{
            \"name\": \"{}\",
            \"includePath\": [
                \"${{workspaceFolder}}/**\"
            ],
            \"defines\": [
                \"_DEBUG\",
                \"UNICODE\",
                \"_UNICODE\"
            ],
            \"compilerPath\": \"{}/{}\",
            \"cStandard\": \"c11\",
            \"cppStandard\": \"c++20\",
            \"intelliSenseMode\": \"{}-x64\"
        }}
    ],
    \"version\": 4
}}",
                env::consts::OS,
                Self::get_compiler_path(project).unwrap(),
                project.get_compiler().get_compiler_command(project.get_language()),
                command
            ),
        );

        self.configs.insert(
            "tasks".to_string(),
            "{
    \"tasks\": [
        {
            \"type\": \"shell\",
            \"label\": \"run\",
            \"command\": \"ocean run\",
            \"problemMatcher\": [
                \"$gcc\"
            ],
            \"group\": {
                \"kind\": \"build\",
                \"isDefault\": true
            }
        },
        {
            \"type\": \"shell\",
            \"label\": \"run (release)\",
            \"command\": \"ocean run\",
            \"args\": [
                \"--release\",
            ],
            \"problemMatcher\": [
                \"$gcc\"
            ],
            \"group\": \"build\",
        },
        {
            \"type\": \"shell\",
            \"label\": \"build\",
            \"command\": \"ocean build\",
            \"problemMatcher\": [
                \"$gcc\"
            ],
            \"group\": \"build\",
            
        },
        {
            \"type\": \"shell\",
            \"label\": \"build (release)\",
            \"command\": \"ocean build\",
            \"args\": [
                \"--release\",
            ],
            \"problemMatcher\": [
                \"$gcc\"
            ],
            \"group\": \"build\",
            
        },
    ],
    \"version\": \"2.0.0\"
}"
            .to_string(),
        );

        self.configs.insert(
            "launch".to_string(),
            format!(
                "{{
    \"version\": \"0.2.0\",
    \"configurations\": [
        {{
            \"name\": \"Debug\",
            \"type\": \"cppdbg\",
            \"request\": \"launch\",
            \"program\": \"${{workspaceFolder}}/build/debug/{}\",
            \"args\": [],
            \"stopAtEntry\": false,
            \"cwd\": \"${{workspaceFolder}}\",
            \"environment\": [],
            \"externalConsole\": false,
            \"MIMode\": \"gdb\",
            \"preLaunchTask\": \"build\",
            \"setupCommands\": [
                {{
                    \"description\": \"Enable pretty-printing for gdb\",
                    \"text\": \"-enable-pretty-printing\",
                    \"ignoreFailures\": true
                }}
            ]
        }}
    ]
}}",
                format!(
                    "{}{}",
                    project.get_name(),
                    if env::consts::OS == "windows" { ".exe" } else { "" }
                )
            ),
        );

        self
    }
}
