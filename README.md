<h1 style="text-align: center"> Ocean - C/C++ Project Manager/Build System</h1>
<h3 style="text-align: center">
    <i>
        Disclaimer: this is a personal project that I spend my free time on.
    </i>
</h3>

Ocean is a project manager, similar to Rust's Cargo, for C and C++ written with
Rust - *that other systems programming language*. The command syntax is very
similar to that of Cargo's.

By default, Ocean will use `gcc` to compile C and C++ source files, outputting
the executables to `build/{mode}/{project_name}`.

## Table of contents
1. [How to install](#how-to-install)
1. [Requirements](#requirements)
1. [Features](#features)
1. [Command help](#command-help)
    - [`build`](#build)
    - [`clean`](#clean)
    - [`get`](#get)
    - [`set`](#set)
    - [`new`](#new)
    - [`run`](#run)
1. [FAQ](#faq)
    1. [Are you making a package
       manager?](#q%3A-are-you-making-a-package-manager%3F)
    1. [How often do you plan on working on
       Ocean?](#q%3A-how-often-do-you-plan-on-working-on-ocean%3F)
    1. [Why use this over CMake or
       Premake?](#q%3A-why-use-this-over-cmake%2C-premake-(etc.)%3F)
1. [ToDo](#to-do)

## Requirements
- `rustc >= 1.46.0`
- `gcc` (techincally optional if you specify another compiler).
- `g++` (techincally optional if you specify another compiler).


## How to install

1) Make sure you have Cargo and Rust installed: https://rustup.rs/.
2) From a command line, enter the following command: `cargo install --git
https://github.com/STBoyden/ocean`.
3) Done!

Make sure to use `ocean --help` if you're not sure how to use this application.

## Features
- The ability to build and run your project with a single command.
- Easy syntax - designed to be similar to Rust's Cargo.
- Easy setup - all the available options that can be changed in Ocean.toml can
  also be changed through the commands.
- Small project preparation time - can get your C/C++ project up and running in
  only a few seconds (with optional config arguments for multiple editors of your choice).

## Command help
```
Usage: ocean [OPTION]

Create and manage C and C++ projects.

    build           Builds the current project
    clean           Cleans the current project's build artifacts
    get             Returns the values set in the Ocean.toml
    set             Sets the values inside Ocean.toml
    help, --help    Shows this help text
    new             Creates a new C/C++ project in a new directory
    run             Runs the current project, builds if no build is present
```

#### `build`
```
Usage: ocean build [OPTIONS]

By default, this builds projects in debug mode.

Options:
    -d, --debug     Builds the current project in debug mode (this is turned on by default)
    -r, --release   Builds the current project in release mode
    -v, --verbose   Makes the compiler output verbose.
```

#### `get`
```
Usage: ocean get [KEY]

This gets the current values inside the Ocean project file related to a datakey entered by the user.

Option:
    build_dir                       Prints the build directory for the current project.
    c++_compiler, cxx_compiler      Prints the compiler being used for the C++ project.
    c_compiler                      Prints the compiler being used for the C project.
    compiler, current_compiler      Prints the current compiler being used for the project.
    lang, language                  Prints the current language of the project.
    lib_dirs, library_directories   Prints the library directories that would be searched by the linker.
    libs, libraries                 Prints the libraries being compiled with the project.
    name                            Prints the name of the project.
    object_dir                      Prints the object output directory.
    source_dir                      Prints the source code directory.
```

#### `set`
```
Usage: ocean set [KEY]

This set values inside the Ocean project file to a value specified by the user.

Option:
    build_dir [DIRECTORY]                               Sets the build directory for the project.
    c++_compiler [COMPILER], cxx_compiler [COMPILER]    Set the compiler being used for the C++ project.
    c_compiler [COMPILER]                               Sets the compiler being used for the C project.
    compiler [COMPILER], current_compiler [COMPILER]    Sets the current compiler being used for the project.
    lang [LANG], language [LANG]                        Set the current language of the project.
    lib_dirs [DIRS], library_directories [DIRS]         Sets the library directories that would be searched by the linker, split by commas.
    libs [LIBS], libraries [LIBS]                       Sets the libraries being compiled with the project, split by commas.
    name [NAME]                                         Sets the name of the project.
    object_dir [DIRECTORY]                              Sets the object output directory.
    source_dir [DIRECTORY]                              Sets the source code directory.
```

#### `new`
```
Usage: ocean new [NAME] [OPTIONS]

This creates a new project with a generated Ocean.toml in a new directory with a specified NAME.
Options:
    -C                  Creates a new C project (default).
    -CXX                Creates a new C++ project.
    -b, --build-dir     Sets the build directory (default is "./build")
    -s, --source-dir    Sets the source directory (default is "./src")
    -o, --obj-dir       Sets the objects directory (default is "./obj")
    -c, --compiler      Sets the compiler for the current project (default is gcc for C and g++ for C++).
    --ccls              Outputs a .ccls file to be used with ccls. Allows a language server to be used with an editor like Vim, for example.
    --vscode            Outputs Visual Studio Code config files to make writing C/C++ easier.
```

#### `run`
```
Usage: ocean run [OPTIONS]

By default, this run projects in debug mode.

Options:
    -d, --debug     Runs the current project in debug mode (this is turned on by default)
    -r, --release   Runs the current project in release mode
    -v, --verbose   Makes the compiler output verbose.
```


## FAQ

#### Q: Are you making a package manager?
A: Not *yet*. I am considering making a package manager but I have not decided
fully. Either way, this will be decided at  a later date.

#### Q: How often do you plan on working on Ocean?
A: Seeing as this is a personal side project, I will spend time on it when I
please but probably quite often. I do have a job so that will take priority.

#### Q: Why use this over CMake, Premake (etc.)?
A: At least for me personally, using Ocean takes less time to get things set up
over something like CMake and Premake. Especially for smaller projects where I
just want to prototype something quickly and easily without having to mess
around in a CMakeLists.txt. However, Ocean is **not** a replacement for either
CMake or Premake and is not intended to.

## To Do

- [ ] Use `cc` crate instead of manually calling the compiler commands.
- [ ] Use `clap` or `structopt` to parse arguments.
- [ ] Work on incremental builds:
    - Look into Go's build cache.
- [ ] Provide examples of Ocean usage with varying degree of project size.

