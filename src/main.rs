#![allow(clippy::pedantic)]

mod cache;
mod commands;
mod compiler;
mod editors;
mod language;
mod platform;
mod project;

use commands::*;
use std::env;

fn parse_args(mut args: Vec<String>) -> Result<(), String> {
    let platforms = ["linux", "osx", "windows"];

    if args[1..].is_empty() {
        help(None);
        return Err("No arguments were specified".to_string());
    } else {
        args = args[1..].to_vec();
    }

    match args[0].as_str() {
        "build" => build(&args[1..])?,
        "clean" => clean()?,
        "get" =>
            if platforms.contains(&args[1].as_str()) {
                get_data_platform(&args[2..], args[1].clone())?;
            } else {
                get_data(&args[1..])?;
            },
        "help" | "--help" => help(None),
        "new" => new(&args[1..])?,
        "run" => run(&args[1..])?,
        "set" =>
            if platforms.contains(&args[1].as_str()) {
                set_data_platform(&args[2..], args[1].clone())?;
            } else {
                set_data(&args[1..])?;
            },
        _ => help(Some(&args[0])),
    };

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Err(e) = parse_args(args) {
        eprintln!("Error: '{}'", e);
    }
}
