use crate::project::Project;
use std::{fs::File, io::prelude::*};

pub fn write_ignore(project: &Project) {
    let ignore_content = "/build/\n/obj/\nOcean.lock";
    let mut ignore_file =
        File::create(&format!("{}/.gitignore", project.get_name())).expect("Could not create .gitignore");
    ignore_file
        .write_all(ignore_content.as_bytes())
        .expect("Could not write into .gitignore");
}
