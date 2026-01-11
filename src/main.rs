use crate::errors::*;
use crate::session::Session;
use crate::vault_entry_manager::*;
use crate::vault_file_manager::*;
use clap::{Parser};
use cli::*;
use password_manager::active_session;
use std::io::{self, Write};
use std::time::Duration;
use std::thread;


fn main() {
    password_manager::intro_animation();
    let mut current_session: Option<Session> = None;
    let mut current_vault: Option<Vault>     = None;

    'interactive_shell: loop {
        //println!("===================");
        println!("___________________");
        println!("Current vault: {}", 
            match current_vault {
                Some(v) => v.get_name(),
                None    => "None"
            }
        );
        println!("What action do you want to do? ");
        
        if !check_vaults_exist() {
            eprintln!("\nHint: There are currently no vaults at all, consider using 'init' to create one!");
        } else if current_vault.is_none() {
            eprintln!("\nHint: There are currently no vaults open, consider using 'open <vault-name>'!");
        }
        
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // split input like CLI-Args
        let args: Vec<String> = input
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        // Clap expects args including program names as args[0]
        let mut args_with_prog = vec!["pw".to_string()];
        args_with_prog.extend(args);

        // parse input with parse
        let cli = match CLI::try_parse_from(args_with_prog) {
            Ok(cli)  => cli,
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };

        match cli.command {
            CommandCLI::Init { name } => {
                if !active_session(&current_session) {
                    println!("There is no session active right now, consider using open <vault-name>!");
                    continue 'interactive_shell;
                }
                match handle_command_init(name) {
                    Ok(())            => {
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
                if !active_session(&current_session) {
                    println!("There is no session active right now, consider using open <vault-name>!");
                    continue 'interactive_shell;
                }
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
                if !active_session(&current_session) {
                    println!("There is no session active right now, consider using open <vault-name>!");
                    continue 'interactive_shell;
                }
                match handle_command_get(&mut current_vault, name, show) {
                    Ok(())             => {/* Do Nothing */},
                    Err(VaultError::NoVaultOpen) => {
                        println!("No vault is active! Use init or open <vault-name>!");
                    }
                    Err(VaultError::CouldNotGetEntry) => {
                        // because of printing the name of non-existent vault => in cli.rs
                        /* Do Nothing */
                    }
                    Err(VaultError::AnyhowError(ref e)) if e.to_string() == "Exit" => {
                        println!("Exiting...");
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
                continue 'interactive_shell;
            },

            CommandCLI::Getall { show  } => {
                if !active_session(&current_session) {
                    println!("There is no session active right now, consider using open <vault-name>!");
                    continue 'interactive_shell;
                }
                match handle_command_getall(&mut current_vault, show) {
                    Ok(()) => {/* Do nothing */},
                    Err(VaultError::NoVaultOpen) => {
                        println!("No vault is active! Use init or open <vault-name>!");
                    }
                    Err(VaultError::InvalidKey) => {
                        println!("The given password is incorrect!");
                    }
                    Err(VaultError::CouldNotGetEntry) => {
                        println!();
                        println!("The current vault does not have any entries yet!");
                        println!("Hint: Use add to create his first one!");
                    }
                    Err(VaultError::AnyhowError(ref e)) if e.to_string() == "Exit" => {
                        println!("Exiting...");
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
                continue 'interactive_shell;
            },

            CommandCLI::Delete { name } => {
                if !active_session(&current_session) {
                    println!("There is no session active right now, consider using open <vault-name>!");
                    continue 'interactive_shell;
                }
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

            CommandCLI::Deletevault {  } => {
                // There is a Anyhow error here, if current_vault == None, no need to check active session
                match handle_command_deletevault(&mut current_vault) {
                    Ok(()) => {
                        current_vault = None;
                    }
                    Err(VaultError::AnyhowError(ref e)) if e.to_string() == "Cancelled." => {
                        println!("\nDeletion cancelled.");
                    }
                    Err(VaultError::AnyhowError(ref e)) if e.to_string() == "exit" => {
                        println!("Exiting...");
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
                continue 'interactive_shell;
            }

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

            CommandCLI::ChangeMaster {  } => {
                match handle_command_change_master(&mut current_vault) {
                    Ok(()) => {/* Do nothing */}
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
                continue 'interactive_shell;
            },

            CommandCLI::Edit { name } => {
                match handle_command_edit(&mut current_vault, name) {
                    Ok(()) => {/* Do nothing */}
                    Err(e) => {
                        println!("Error: {}", e)
                    }
                }
                continue 'interactive_shell;
            },

            CommandCLI::Open { name } => {
                match handle_command_open(name, &current_session, &current_vault) {
                    Ok(session) => {
                        current_session = Some(session);
                        current_vault = session.opened_vault;
                    },
                    Err(VaultError::InvalidKey) => {
                        println!("Error: Invalid password!")
                    }
                    Err(e) => {
                        println!("Error opening vault: {}", e);
                    }
                }
            },

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
