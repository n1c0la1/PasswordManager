mod cli;
mod vault_manager;
mod errors;

use crate::errors::*;
use clap::{Parser, Command, Subcommand, command};
use serde_json::Value; //imports value type (represents json data)
use std::fs::{self, read}; //imports rusts file system module
use cli::*;
use vault_manager::*;
use std::path::{Path, PathBuf};
use std::io::{self, Read, Write};
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};
use std::thread;


fn main() {
    password_manager::intro_animation();
    let mut current_vault: Option<Vault> = None;

    let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["|", "/", "-", "\\"])
                .template("{spinner} {msg}")
                .unwrap(),
        );

    'interactive_shell: loop {
        //println!("===================");
        println!("___________________");
        println!("Current vault: {}", 
            match current_vault.as_ref() {
                Some(v) => v.get_name(),
                None    => "None"
            }
        );
        println!("What action do you want to do? ");
        
        if !check_vaults_exist() {
            eprintln!("\nHint: There are currently no vaults at all, consider using 'init' to create one!");
        } else if current_vault.is_none() {
            eprintln!("There are currently no vaults open, consider using 'open <vault-name>'!")
        }
        
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // Die Eingabe splitten wie CLI-Args
        let args: Vec<String> = input
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        // Clap erwartet args inklusive Programmnamen als args[0]
        let mut args_with_prog = vec!["pw".to_string()];
        args_with_prog.extend(args);

        // Mit Clap parsen
        let cli = match CLI::try_parse_from(args_with_prog) {
            Ok(cli) => cli,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };
        match cli.command {
            CommandCLI::Init { name } => {
                match handle_command_init(name) {
                    Ok(vault)            => {current_vault = Some(vault);},
                    Err(VaultError::NameExists) => {
                        println!();
                        println!("Error: {}", VaultError::NameExists);
                        println!("Use a different name or open the existing vault.");
                        println!();},
                    Err(e)          => {println!("Error: {e}")},
                }
                continue 'interactive_shell;
            },

            CommandCLI::Add { name, username, url, notes , password} => {
                handle_command_add(&mut current_vault, name, username, url, notes, password);
            },

            CommandCLI::Get { name, show } => {
                handle_command_get(&mut current_vault, name, show);
            },

            CommandCLI::ShowEntries { vault, show  } => todo!(),

            CommandCLI::Delete { name } => todo!(),

            CommandCLI::Generate { length, no_symbols } => todo!(),

            CommandCLI::ChangeMaster {  } => todo!(),

            CommandCLI::Modify { name } => todo!(),

            CommandCLI::Open { name } => {todo!()},

            CommandCLI::Switch { name } => todo!(),

            CommandCLI::Vaults {  } => {handle_command_vaults(&current_vault);},

            CommandCLI::Clear {  } => {handle_command_clear();},

            CommandCLI::Quit { force } => { 
                match handle_command_quit(current_vault.clone(), force) {
                    LoopCommand::Break    => {
                        break    'interactive_shell;},
                    LoopCommand::Continue => {
                        thread::sleep(Duration::from_millis(500));
                        continue 'interactive_shell;}
                }
            },
        }
    }
}
