mod cli;

use clap::{Parser, Command, Subcommand, command};
use serde_json::Value; //imports value type (represents json data)
use std::fs; //imports rusts file system module
use cli::*;
use std::path::PathBuf;


fn main() {
    let string_from_json =
        fs::read_to_string("src/passwords_file.json").expect("could not read file");
    let json_data: Value = serde_json::from_str(&string_from_json).expect("invalid json");

    println!("{json_data}");

    password_manager::intro_animation();
    println!("Hello, world!");

    // loop can only start after vault is initialized and master password is verified
    // if no vault exists, prompt to create and name one
    // if more than one vault exists, prompt to select one
    // this vault = selected vault
    // prompt for master password and verify
    // unklar wie vault funktionieren soll, deshalb erstmal nur für die JSON Datei
    if !PathBuf::from("src/passwords_file.json").exists() {
        println!("No vault found, please initialize a new vault.");
        // Call init
    } else {
        println!("Vault found, please enter your master password.");
        // Verify password
        
    }

    loop {
        println!("What action do you want to do?");

        let input = CLI::parse();
        match input.command {
            CommandCLI::Init {} => todo!(),
            CommandCLI::Add { name, username, url, notes , password} => todo!(),
            CommandCLI::Get {  } => todo!(),
            CommandCLI::List {  } => todo!(),
            CommandCLI::Remove {  } => todo!(),
            CommandCLI::Generate {  } => todo!(),
            CommandCLI::ChangeMaster {  } => todo!(),
            CommandCLI::Modify {  } => todo!(),
            CommandCLI::Quit {  } => break,
        }
    }
}
