mod cli;
use cli::CLI;
use clap::{Parser, Subcommand, command};
use std::path::PathBuf;
use rpassword;


fn main() {
    password_manager::intro_animation();
    println!("Hello, world!");

    let input: CLI = CLI::parse();
}
