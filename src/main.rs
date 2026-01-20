mod cli;
mod errors;
mod vault_entry_manager;
mod vault_file_manager;
mod crypto;
mod session;

use crate::errors::*;
use crate::session::Session;
use crate::vault_entry_manager::*;
use crate::vault_file_manager::*;
use crate::session::*;
use clap::{Parser};
use cli::*;
use std::io::{self, Write};
use std::time::Duration;
use std::thread;


fn main() {
    intro_animation();
    let mut current_session: Option<Session> = None;
    let mut current_vault: Option<Vault>     = None;

    'interactive_shell: loop {
        //println!("===================");
        println!("___________________");
        println!("Current vault: {}", 
            match &mut current_vault {
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
                    // nothing needed to do, vault gets created and closed immidiatly
                    /* Do nothing */
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
                    Ok(())             => {
                        // write changes from current_vault to current_session with save
                        match &mut current_session {
                            Some(session) => {
                                session.new_save(&current_vault);
                            }
                            None => {
                                // Should never happen because of active_session check
                                println!("Something went wrong, try starting a new session");
                            }
                        }
                    },
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
                match handle_command_get(&mut current_session, &mut current_vault, name, show) {
                    Ok(())             => {/* Do nothing, vault did not change */}
                    Err(SessionError::VaultError(VaultError::NoVaultOpen)) => {
                        println!("No vault is active! Use init or open <vault-name>!");
                    }
                    Err(SessionError::VaultError(VaultError::CouldNotGetEntry)) => {
                        // already printing the name of non-existent vault in cli.rs
                        /* Do Nothing */
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
                match handle_command_getall(&mut current_session, &mut current_vault, show) {
                    Ok(()) => {/* Do nothing, vault did not change */}
                    Err(SessionError::VaultError(VaultError::NoVaultOpen)) => {
                        println!("No vault is active! Use init or open <vault-name>!");
                    }
                    Err(SessionError::VaultError(VaultError::CouldNotGetEntry)) => {
                        println!();
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
                if !active_session(&current_session) {
                    println!("There is no session active right now, consider using open <vault-name>!");
                    continue 'interactive_shell;
                }
                match handle_command_delete(&mut current_vault, name) {
                    Ok(()) => {
                        // write changes from current_vault to current_session with save
                        match &mut current_session {
                            Some(session) => {
                                session.new_save(&current_vault);
                                println!("Vault saved.\n");
                            }
                            None => {
                                // Should never happen because of active_session check
                                println!("Something went wrong, try starting a new session");
                            }
                        }
                    }
                    Err(SessionError::VaultError(VaultError::NoVaultOpen)) => {
                        println!("No vault is active! Use init or open <vault-name>!");
                    }
                    Err(SessionError::VaultError(VaultError::EntryNotFound)) => {
                        // because of printing name => in cli.rs
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
                continue 'interactive_shell;
            },

            CommandCLI::Deletevault {  } => {
                if !active_session(&current_session) {
                    println!("Due to RustPass's logic, you have to open your vault first!");
                    println!("Hint: Consider using open <vault-name>!");
                    continue 'interactive_shell;
                }
                match handle_command_deletevault(&mut current_session, &mut current_vault) {
                    Ok(()) => {
                        current_vault   = None;
                        current_session = None;
                    }
                    Err(SessionError::VaultError(VaultError::AnyhowError(ref e))) if e.to_string() == "Cancelled." => {
                        println!("\nDeletion cancelled.");
                    }
                    Err(SessionError::VaultError(VaultError::AnyhowError(ref e))) if e.to_string() == "exit" => {
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
                if !active_session(&current_session) {
                    println!("There is no session active right now, consider using open <vault-name>!");
                    continue 'interactive_shell;
                }
                match handle_command_change_master(&mut current_session) {
                    Ok(()) => {/* Do nothing, saving and closing and ending session happens in cli.rs */}
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
                continue 'interactive_shell;
            },

            CommandCLI::Edit { name } => {
                if !active_session(&current_session) {
                    println!("There is no session active right now, consider using open <vault-name>!");
                    continue 'interactive_shell;
                }
                match handle_command_edit(&mut current_vault, name) {
                    Ok(()) => {
                        // write changes from current_vault to current_session with save
                        match &mut current_session {
                            Some(session) => {
                                session.new_save(&current_vault);
                            }
                            None => {
                                // Should never happen because of active_session check
                                println!("Something went wrong, try starting a new session");
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e)
                    }
                }
                continue 'interactive_shell;
            },

            CommandCLI::Open { name } => {
                match handle_command_open(name, &mut current_session) {
                    Ok(session) => {
                        if session.opened_vault.is_none() {
                            println!("Something went wrong!"); 
                            continue 'interactive_shell;
                        }
                        current_session = Some(session);
                    },
                    Err(SessionError::VaultError(VaultError::InvalidKey)) => {
                        println!("Error: Invalid password!")
                    }
                    Err(e) => {
                        println!("Error opening vault: {}", e);
                    }
                }
                continue 'interactive_shell;
            },
            CommandCLI::Close { force } => {
                if !active_session(&current_session) {
                    println!("There is no session active right now, consider using open <vault-name>!");
                    continue 'interactive_shell;
                }
                match handle_command_close(&mut current_session, force) {
                    Ok(LoopCommand::Continue) => {
                        // if the user says yes to closing.
                        current_session = None;
                        current_vault   = None;
                    }
                    Ok(LoopCommand::Cancel) => {
                        // if the user wishes to cancel the closing process.
                        /* Do nothing */
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }
                continue 'interactive_shell;
            },

            CommandCLI::Vaults {  } => {handle_command_vaults(&current_vault);},

            CommandCLI::Clear {  } => {handle_command_clear();},

            CommandCLI::Quit { force } => { 
                match handle_command_quit(force) {
                    Ok(LoopCommand::Continue)    => {
                        if let Some(mut session) = current_session {
                            match session.end_session() {
                                Ok(()) => {/* Do nothing */},
                                Err(e) => {
                                    println!("Error: {}", e);
                                }
                            }
                        }
                        break    'interactive_shell;},
                    Ok(LoopCommand::Cancel) => {
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
