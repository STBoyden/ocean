use crate::project::Project;
use serde_derive::*;
use std::{
    collections::hash_map::DefaultHasher,
    fs::*,
    hash::*,
    io::prelude::*,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize, Serialize)]
struct FileData {
    pub path: PathBuf,
    pub hash: String,
}

#[derive(Deserialize, Serialize)]
pub struct Cache {
    files: Vec<FileData>,
}

impl Cache {
    fn get_dir_contents(path: PathBuf) -> Option<Vec<PathBuf>> {
        let path = Path::new(&path);

        if path.is_dir() {
            let mut contents = vec![];
            let dir_contents = read_dir(path).expect("Could not read directory");

            for item in dir_contents {
                let item = item.unwrap();

                if item.path().is_dir() {
                    contents.append(&mut Self::get_dir_contents(item.path()).unwrap());
                } else {
                    contents.push(item.path());
                }
            }

            Some(contents)
        } else {
            None
        }
    }

    fn get_all_files(project: &Project) -> Result<Vec<FileData>, String> {
        let dir_contents: Vec<PathBuf> = match read_dir(project.get_directories().get_source_dir()) {
            Ok(cont) => cont,
            Err(e) =>
                return Err(format!(
                    "Could not read source directory (\"{}\"): {}",
                    project.get_directories().get_source_dir(),
                    e
                )),
        }
        .into_iter()
        .map(|x| x.unwrap().path())
        .collect();

        let mut paths = vec![];
        dir_contents.iter().for_each(|x| {
            paths.append(&mut match Self::get_dir_contents(x.clone()) {
                Some(v) => v,
                None => vec![x.clone()],
            })
        });

        let mut files = vec![];
        for item in paths {
            let file_data = File::open(item.clone());
            let mut buffer = Vec::<u8>::new();

            file_data
                .unwrap()
                .read_to_end(&mut buffer)
                .expect("Could not read file.");

            files.push(FileData {
                path: item.clone(),
                hash: {
                    let mut s = DefaultHasher::new();
                    s.write(&buffer[..]);
                    format!("{:x}", s.finish())
                },
            });
        }

        Ok(files)
    }

    pub fn new(project: &Project) -> Result<Self, String> {
        Ok(Self {
            files: Self::get_all_files(project)?,
        })
    }

    pub fn get_changed(&self, project: &Project) -> Result<Vec<PathBuf>, String> {
        if !Path::new("Ocean.lock").exists() {
            return Err("Cannot find Ocean.lock in project root.".to_string());
        }

        let mut changed = vec![];
        let current_files = Self::get_all_files(project)?;
        let diffs = self.files.iter().zip(current_files.iter());

        for (old_diff, curr_diff) in diffs {
            if old_diff.hash != curr_diff.hash {
                changed.push(curr_diff.path.clone());
            }
        }

        Ok(changed)
    }

    pub fn update_cache<'a>(&mut self, project: &Project) -> Result<(), &'a str> {
        if !Path::new("Ocean.lock").exists() {
            return Err("Cannot find Ocean.lock in project root.");
        }

        self.files = Self::get_all_files(project).unwrap();

        Ok(())
    }
}
