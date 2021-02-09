#![allow(clippy::pedantic)]

mod cache;
mod commands;
mod common;
mod compiler;
mod editors;
mod language;
mod platform;
mod project;

#[macro_use]
extern crate serde;

use commands::Commands;
use common::StrRet;
use std::env;

fn parse_args(mut args: Vec<String>) -> Result<(), StrRet> {
    let platforms = ["linux", "osx", "windows"];

    if args[1..].is_empty() {
        Commands::help(None);
        return Err("No arguments were specified".into());
    } else {
        args = args[1..].to_vec();
    }

    match args[0].as_str() {
        "build" => Commands::build(&args[1..])?,
        "clean" => Commands::clean()?,
        "get" =>
            if !args[1..].is_empty() && platforms.contains(&args[1].as_str()) {
                Commands::get_data_platform(&args[2..], args[1].clone())?;
            } else {
                Commands::get_data(&args[1..])?;
            },
        "help" | "--help" => Commands::help(None),
        "new" => Commands::new_project(&args[1..])?,
        "run" => Commands::run(&args[1..])?,
        "set" =>
            if !args[1..].is_empty() && platforms.contains(&args[1].as_str()) {
                Commands::set_data_platform(&args[2..], args[1].clone())?;
            } else {
                Commands::set_data(&args[1..])?;
            },
        _ => Commands::help(Some(&args[0])),
    };

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if let Err(e) = parse_args(args) {
        eprintln!("Error: {}", e);
    }
}
