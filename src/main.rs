mod cli;
mod vault_manager;

use clap::{Parser, Command, Subcommand, command};
use serde_json::Value; //imports value type (represents json data)
use std::fs; //imports rusts file system module
use cli::*;
use vault_manager::*;
use std::path::{Path, PathBuf};
use std::io::{self, Write};
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};


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
                    Ok(vault) => {current_vault = Some(vault);},
                    Err(VaultError::NameExists) => {
                        println!();
                        println!("Error: {}", VaultError::NameExists);
                        println!("Use a different name or open the existing vault.");
                        println!();},
                    Err(e) => {println!("Error: {e}")},
                }

                continue 'interactive_shell;
            },

            CommandCLI::Add { name, username, url, notes , password} => {
                if !check_vaults_exist() {
                    eprintln!("There are currently no vaults, consider using 'init' to create one!");
                    continue 'interactive_shell;
                }

                // Auto-open vault if not open
                if !ensure_vault_open(&mut current_vault) {
                    continue;
                }
                
                if let Some(ref mut vault) = current_vault {
                    let pw = if let Some(p) = password {
                        Some(p)
                    } else {
                        print!("Enter password for the entry (or press Enter to skip): ");
                        io::stdout().flush().unwrap();
                        let input_pw = rpassword::read_password().unwrap();
                        if input_pw.is_empty() {
                            None
                        } else {
                            Some(input_pw)
                        }
                    };
                    
                    // TODO when no email provided, ask the user.

                    let entry = Entry::new(name.clone(), username, pw, url, notes);
                    
                    match vault.add_entry(entry) {
                        Ok(_) => {

                            spinner.enable_steady_tick(Duration::from_millis(80));
                            spinner.set_message("Adding PasswordEntry...");

                            vault.save();
                            println!("Entry '{}' added successfully!", name);
                            println!("Vault saved.\n");

                            spinner.finish_and_clear();
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                }
            },
            CommandCLI::Get { name, show } => todo!(),
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
                if force {
                    println!("Quitting RustPass...");
                    break 'interactive_shell;
                } 

                print!("Are you sure you want to quit? (y/n): ");
                io::stdout().flush().unwrap();


                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();

                if input.trim().eq_ignore_ascii_case("y") {
                    println!("Quitting RustPass...");
                    if current_vault.is_some() {
                        current_vault.unwrap().close();
                    }
                    break 'interactive_shell;
                } else {
                    println!("Cancelled. \n");
                    io::stdout().flush().unwrap();
                }
            },
        }
    }
}
