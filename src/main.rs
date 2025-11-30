mod cli;

use clap::Parser;
use serde_json::Value; //imports value type (represents json data)
use std::fs; //imports rusts file system module
use cli::CLI;
use std::path::PathBuf;


fn main() {
    let string_from_json =
        fs::read_to_string("src/passwords_file.json").expect("could not read file");
    let json_data: Value = serde_json::from_str(&string_from_json).expect("invalid json");

    println!("{json_data}");

    password_manager::intro_animation();
    println!("Hello, world!");

    let input: CLI = CLI::parse();
}
