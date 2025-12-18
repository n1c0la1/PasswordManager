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
            eprintln!("\nThere are currently no vaults open, consider using 'open <vault-name>'!");
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
            Ok(cli)  => cli,
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };

        match cli.command {
            CommandCLI::Init { name } => {
                match handle_command_init(name) {
                    Ok(vault)            => {
                        if let Some(opened_vault) = current_vault.take() {
                            match opened_vault.close() {
                                Ok(())             => {/*  Do nothing */},
                                Err(e) => {
                                    println!("Error: {}", e);
                                }
                            }
                        }

                        current_vault = Some(vault);
                    },
                    Err(VaultError::NameExists) => {
                        println!();
                        println!("Error: {}", VaultError::NameExists);
                        println!("Use a different name or open the existing vault.");
                        println!();
                    },
                    Err(e)          => {
                        println!("Error: {e}");
                    },
                }
                continue 'interactive_shell;
            },

            CommandCLI::Add { name, username, url, notes , password} => {
                match handle_command_add(&mut current_vault, name, username, url, notes, password) {
                    Ok(())             => {/* Do Nothing */},
                    Err(VaultError::NoVaultOpen) => {
                        println!("Error: {}", VaultError::NoVaultOpen);
                        println!("Consider using init or open <vault-name>!");
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
                continue 'interactive_shell;
            },

            CommandCLI::Get { name, show } => {
                match handle_command_get(&mut current_vault, name, show) {
                    Ok(())             => {/* Do Nothing */},
                    Err(VaultError::NoVaultOpen) => {
                        println!("No vault is active! Use init or open <vault-name>!");
                    }
                    Err(VaultError::CouldNotGetEntry) => {
                        // because of printing the name of non-existent vault => in cli.rs
                        /* Do Nothing */
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
                continue 'interactive_shell;
            },

            CommandCLI::Getall { show  } => {
                match handle_command_getall(&mut current_vault, show) {
                    Ok(()) => {/* Do nothing */},
                    Err(VaultError::NoVaultOpen) => {
                        println!("No vault is active! Use init or open <vault-name>!");
                    }
                    Err(VaultError::InvalidKey) => {
                        println!("The given password is incorrect!");
                    }
                    Err(VaultError::CouldNotGetEntry) => {
                        println!("The current vault does not have any entries yet!");
                        println!("Hint: Use add to create his first one!");
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
                continue 'interactive_shell;
            },

            CommandCLI::Delete { name } => {
                match handle_command_delete(&mut current_vault, name) {
                    Ok(()) => {/* Do Nothing */}
                    Err(VaultError::NoVaultOpen) => {
                        println!("No vault is active! Use init or open <vault-name>!");
                    }
                    Err(VaultError::CouldNotGetEntry) => {
                        // because of printing name => in cli.rs
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
                continue 'interactive_shell;
            },

            CommandCLI::Generate { length, no_symbols } => {
                match handle_command_generate(length, no_symbols) {
                    Ok(generated_pw) => {
                        println!("{}", generated_pw)
                    }
                    Err(e) => {
                        println!("Error: {}", e)
                    }
                }
                continue 'interactive_shell;
            },

            CommandCLI::ChangeMaster {  } => todo!(),

            CommandCLI::Modify { name } => todo!(),

            CommandCLI::Open { name } => {
                match handle_command_open(&mut current_vault, name) {
                    Ok(opened_vault) => {
                        current_vault = Some(opened_vault)
                    },
                    Err(VaultError::InvalidKey) => {
                        println!("Error: Invalid password!")
                    }
                    Err(e) => {
                        println!("Error opening vault: {}", e);
                    }
                }
            },

            CommandCLI::Switch { name } => todo!(),

            CommandCLI::Vaults {  } => {handle_command_vaults(&current_vault);},

            CommandCLI::Clear {  } => {handle_command_clear();},

            CommandCLI::Quit { force } => { 
                match handle_command_quit(force) {
                    Ok(LoopCommand::Break)    => {
                        if let Some(opened_vault) = current_vault {
                            match opened_vault.close() {
                                Ok(()) => {/* Do nothing */},
                                Err(e) => {
                                    println!("Error: {}", e);
                                }
                            }
                        }
                        break    'interactive_shell;},
                    Ok(LoopCommand::Continue) => {
                        thread::sleep(Duration::from_millis(500));
                        continue 'interactive_shell;}
                    Err(e) => {
                        println!("Error: {}", e);
                        continue 'interactive_shell;
                    }
                }
            },
        }
    }
}
                    