use crate::errors::*;
//use password_manager::*;
use crate::vault_entry_manager::*;
use crate::session::*;
//use crate::vault_file_manager::initialize_vault;
//use crate::test::*;

use anyhow::anyhow;
use clap::{Parser, Subcommand, command};
use indicatif::{self, ProgressBar, ProgressStyle};
use rpassword;
use secrecy::ExposeSecret;
use secrecy::SecretString;
use std::fs;
use std::io::stdout;
use std::io::{self, Write};
use std::path::Path;
use std::{time::Duration};
use arboard::Clipboard;

#[derive(Parser)]
#[command(name = "pw")]
pub struct CLI {
    #[command(subcommand)]
    pub command: CommandCLI,
}

pub enum LoopCommand {
    Continue,
    Cancel,
}

#[derive(Subcommand)]
pub enum CommandCLI {
    /// Initializes a new PasswordManager.
    Init {
        name: Option<String>,
    },

    /// Adds a new password to database.
    Add {
        name: Option<String>,

        #[arg(short, long)]
        username: Option<String>,

        #[arg(long)]
        url: Option<String>,

        #[arg(short, long)]
        password: Option<String>,

        #[arg(short, long)]
        notes: Option<String>,
    },

    /// Get an Entry of the Database.
    // maybe implement parameter: with or without the password visible
    Get {
        //name of the Element to be shown
        name: String,

        // Specifies whether the password should be shown in the command line.
        #[arg(short = 's', long)]
        show: bool,
        // Maybe an Option to directly copy the password to clipboard without showing it.
    },

    /// Gets all Entries from the current vault.
    // maybe implement filters e.g. all passwords with that email, or on that URL.
    Getall {
        // Show passwords or not.
        #[arg(short = 's', long)]
        show: bool,
    },

    /// Generate a password.
    // maybe implement interaction (abfrage) if with special cases, numbers etc.
    Generate {
        length: i32,

        #[arg(short = 'f', long = "no-symbols")]
        no_symbols: bool,
    },

    /// Remove an entry from Database.
    Delete {
        name: String,
    },

    /// Delete a vault completely.
    // Does not take an argument, because it has to use to opened one.
    Deletevault {},

    /// Change the Masterpassword.
    // implement not visible, old password required. Verschlüsselt Vault sofort
    ChangeMaster {},

    Vaults {},

    /// Modify a given password
    //
    Edit {
        name: String,
    },

    /// Opens given vault.
    Open {
        name: String,
    },

    /// Closes the current vault and ends the session.
    Close {
        #[arg(short = 'f', long = "force")]
        force: bool,
    },

    /// Clears terminal window.
    Clear {},

    /// Quits the input loop.
    Quit {
        //forces quit, normally "Do you really want to quit RustPass?"
        #[arg(short = 'f', long = "force")]
        force: bool,
    },
}

pub fn clear_terminal() {
        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

pub fn intro_animation() {
        let frames =
        r#"
=== RustPass ================================
Secure • Fast • Rust-Powered Password Manager

pub fn handle_command_vaults(current_session: &Option<Session>) {
    println!("\n=== Available Vaults ===");
                
    match fs::read_dir("vaults") {
        Ok(entries) => {
            let mut vault_files: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext == "psdb")
                        .unwrap_or(false)
                })
                .filter_map(|e| {
                    e.path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                })
                .collect();
            
            if vault_files.is_empty() {
                println!("  (no vaults found)");
                println!("\nCreate a new vault with: init <vault_name>");
            } else {
                vault_files.sort();
                
                if current_session.is_none() || !active_session(current_session) {
                    for vault_name in vault_files {
                        println!("    {}", vault_name);
                    }
                }
                else {
                    let current_vault_name = current_session
                        .as_ref()
                        .and_then(|s| s.opened_vault.as_ref())
                        .map(|v| v.get_name());

                    for vault_name in vault_files {
                        if Some(&vault_name) == current_vault_name{
                            println!("  → {} (currently open)", vault_name);
                    } else {
                        println!("    {}", vault_name);
                    }
                    }                    
                }
            }
            println!();
        }
        Err(e) => {
            println!("Error reading vaults directory: {}", e);
        }
    }
}

pub fn handle_command_clear() {
    clear_terminal();
    intro_animation();
}

pub fn handle_command_quit(force: bool) -> Result<LoopCommand, VaultError> {
    if force {
        println!("Quitting RustPass...");
        return Ok(LoopCommand::Continue);
    } 

    print!("Are you sure you want to quit? (y/n): ");
    io::stdout().flush().unwrap();


    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        println!("Quitting RustPass...");
        // Closing the vault is happening in main.rs to avoid cloning.
        Ok(LoopCommand::Continue)
    } else {
        println!("Cancelled. \n");
        io::stdout().flush().unwrap();
        Ok(LoopCommand::Cancel)
    }
}

pub fn copy_to_clipboard(content: &str) -> Result<(), SessionError> {
    let mut clipboard = Clipboard::new().map_err(|_| SessionError::VaultError(VaultError::ClipboardError))?;
    clipboard.set_text(content.to_string()).map_err(|_| SessionError::VaultError(VaultError::ClipboardError))?;
    println!("Passwort wurde in die Zwischenablage kopiert!");
    Ok(())
}

fn add_password_to_entry() -> Result<Option<String>, SessionError> {
    let mut loop_pw = String::new();
                'input_pw: loop {
                    println!("Generate password for entry (y/n): ");
                    print!("> ");
                    let mut input_choice_gen = String::new();
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut input_choice_gen)?;

                    if input_choice_gen.trim().eq_ignore_ascii_case("y") {
                        let length: i32;
                        let no_symbols: bool;

                        'input_length: loop {
                            println!("Enter desired password-length: ");
                            let mut length_input = String::new();
                            io::stdout().flush().unwrap();
                            io::stdin().read_line(&mut length_input)?;
                            if length_input.trim().parse::<i32>().is_ok() {
                                length = length_input.parse::<i32>().unwrap();
                                break 'input_length; 
                            }
                        }
                        
                        print!("Use symbols? (y/n): ");
                        io::stdout().flush().unwrap();
                        let mut no_symbols_input = String::new();
                        io::stdin().read_line(&mut no_symbols_input)?;
                        if input_choice_gen.trim().eq_ignore_ascii_case("y") {
                            no_symbols = false;
                        } else {
                            no_symbols = true;
                        }
                        loop_pw = handle_command_generate(length, no_symbols)?;
                        break 'input_pw;
                    }

                    print!("Enter password for entry (or press Enter to skip): ");
                    io::stdout().flush().unwrap();
                    let input_password = rpassword::read_password()?;
                    
                    if input_password.is_empty() {
                        break 'input_pw;
                    }

                    print!("Please confirm the password: ");
                    io::stdout().flush().unwrap();
                    let confirm = rpassword::read_password()?;
                    
                    if input_password != confirm {
                        println!("The passwords do not match! Try again:");
                        println!();
                        continue 'input_pw;
                    }

                    loop_pw = input_password;
                    break 'input_pw;
                }
                if loop_pw.is_empty() {
                    Ok(None)
                } else {
                    println!();
                    Ok(Some(loop_pw))
                }
}


// tests

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::fs;
//     use std::path::Path;

//     // Helper-function to delete testvault-file
//     fn cleanup_test_vault(vault_name: &str) {
//         let path = format!("vaults/{}.psdb", vault_name);
//         if Path::new(&path).exists() {
//             fs::remove_file(&path).ok();
//         }
//     }

//     // ================== QUIT TESTS ==================
    
//     #[test]
//     fn test_handle_command_quit_with_force() {
//         let result = handle_command_quit(true);
//         assert!(result.is_ok());
//         match result.unwrap() {
//             LoopCommand::Break => assert!(true),
//             LoopCommand::Continue => panic!("Expected Break, got Continue"),
//         }
//     }

//     #[test]
//     fn test_loop_command_variants() {
//         let continue_cmd = LoopCommand::Continue;
//         let break_cmd = LoopCommand::Break;
        
//         match continue_cmd {
//             LoopCommand::Continue => assert!(true),
//             LoopCommand::Break => panic!("Wrong variant for Continue"),
//         }
        
//         match break_cmd {
//             LoopCommand::Break => assert!(true),
//             LoopCommand::Continue => panic!("Wrong variant for Break"),
//         }
//     }

//     // ================== CLEAR TESTS ==================
    
//     #[test]
//     fn test_handle_command_clear() {
//         // test whether no panic
//         handle_command_clear();
//     }

//     // ================== EDIT TESTS ==================
    
//     #[test]
//     fn test_edit_without_vault() {
//         let mut option_vault: Option<Vault> = None;
//         let result = handle_command_edit(&mut option_vault, "test_entry".to_string());
        
//         assert!(result.is_err());
//         match result {
//             Err(VaultError::NoVaultOpen) => assert!(true),
//             _ => panic!("Expected NoVaultOpen error"),
//         }
//     }

//     #[test]
//     fn test_edit_nonexistent_entry() {
//         let vault_name = "test_vault_edit_nonexistent";
        
//         let vault = initialize_vault(vault_name.to_string()).unwrap();
//         vault.save().unwrap();
        
//         let mut option_vault = Some(vault);
        
//         let result = handle_command_edit(&mut option_vault, "nonexistent".to_string());
        
//         assert!(result.is_err());
//         match result {
//             Err(VaultError::EntryNotFound) => assert!(true),
//             _ => panic!("Expected EntryNotFound error"),
//         }
        
//         cleanup_test_vault(vault_name);
//     }

//     #[test]
//     fn test_edit_entry_exists() {
//         let vault_name = "test_vault_edit_exists";
        
//         let mut vault = initialize_vault(vault_name.to_string()).unwrap();
        
//         let entry = Entry::new(
//             "test_entry".to_string(),
//             Some("testuser".to_string()),
//             Some("testpass123".to_string()),
//             Some("https://example.com".to_string()),
//             Some("test notes".to_string()),
//         );
        
//         vault.add_entry(entry).unwrap();
//         vault.save().unwrap();
        
//         let option_vault = Some(vault);
        
//         // check, if test entry is there
//         let vault = option_vault.as_ref().unwrap();
//         assert!(vault.entryname_exists("test_entry"));
        
//         cleanup_test_vault(vault_name);
//     }

//     #[test]
//     fn test_edit_entry_verification() {
//         let vault_name = "test_vault_edit_verification";
        
//         let mut vault = initialize_vault(vault_name.to_string()).unwrap();
        
//         let entry = Entry::new(
//             "test_entry".to_string(),
//             Some("original_user".to_string()),
//             Some("original_pass".to_string()),
//             Some("https://original.com".to_string()),
//             Some("original notes".to_string()),
//         );
        
//         vault.add_entry(entry).unwrap();
//         vault.save().unwrap();
        
//         // close vault and open again
//         vault.close().unwrap();
//         let mut vault = open_vault(vault_name.to_string(), "testkey123".to_string().into()).unwrap();
        
//         // handle edit needs user input -> simulating what happens in handle
//         let entry = vault.get_entry_by_name(&"test_entry".to_string()).unwrap();
//         entry.set_username("modified_user".to_string());
//         entry.set_url("https://modified.com".to_string());
        
//         vault.save().unwrap();
//         vault.close().unwrap();
        
//         // check 
//         let mut vault = open_vault(vault_name.to_string(), "testkey123".to_string().into()).unwrap();
//         let entry = vault.get_entry_by_name(&"test_entry".to_string()).unwrap();
        
//         assert_eq!(*entry.get_user_name(), Some("modified_user".to_string()));
//         assert_eq!(*entry.get_url(), Some("https://modified.com".to_string()));
        
//         cleanup_test_vault(vault_name);
//     }

//     // Tests for getall, generate and delete
//     #[test]
//     fn test_handle_command_getall_no_vault() {
//         let mut opt_vault: Option<Vault> = None;
//         let res = handle_command_getall(&mut opt_vault, false);
//         assert!(res.is_err());
//         match res {
//             Err(VaultError::NoVaultOpen) => {}
//             _ => panic!("Expected NoVaultOpen"),
//         }
//     }

//     #[test]
//     fn test_getall_empty_vault() {
//         let name = "test_vault_getall_empty";
//         let vault = initialize_vault(name.to_string()).unwrap();
//         let mut opt = Some(vault);

//         let res = handle_command_getall(&mut opt, false);
//         assert!(res.is_err());
//         match res {
//             Err(VaultError::CouldNotGetEntry) => {}
//             _ => panic!("Expected CouldNotGetEntry for empty vault"),
//         }

//         cleanup_test_vault(name);
//     }

//     #[test]
//     fn test_getall_success() {
//         let name = "test_vault_getall_success";
//         let mut vault = initialize_vault(name.to_string()).unwrap();
//         let entry = Entry::new(
//             "test_entry".to_string(),
//             Some("testuser".to_string()),
//             Some("testpass123".to_string()),
//             Some("https://example.com".to_string()),
//             Some("test notes".to_string()),
//         );
//         vault.add_entry(entry).unwrap();
//         vault.save().unwrap();
//         let mut opt = Some(vault);
//         let res = handle_command_getall(&mut opt, false);
//         assert!(res.is_ok());

//         cleanup_test_vault(name);
//     }


//     #[test]
//     fn test_generate_invalid_length() {
//         let res = handle_command_generate(0, true);
//         assert!(res.is_err());
//         match res {
//             Err(VaultError::InvalidLength) => {}
//             _ => panic!("Expected InvalidLength for length 0"),
//         }
//     }

//     #[test]
//     fn test_generate_success() {
//         let length = 12;
//         let res = handle_command_generate(length, false);
//         assert!(res.is_ok());
//         let password = res.unwrap();
//         assert_eq!(password.len(), length as usize);
//     }

//     #[test]
//     fn test_generate_correct_characters() {
//         let length = 20;
//         let res = handle_command_generate(length, true);
//         assert!(res.is_ok());
//         let password = res.unwrap();
//         assert_eq!(password.len(), length as usize);
//         for c in password.chars() {
//             assert!(c.is_ascii_alphanumeric(), "Password contains non-alphanumeric character: {}", c);
//         }
//     }


//     #[test]
//     fn test_delete_entry_not_found() {
//         let name = "test_vault_delete_not_found";
//         let vault = initialize_vault(name.to_string()).unwrap();
//         let mut opt = Some(vault);

//         let res = handle_command_delete(&mut opt, "does_not_exist".to_string());
//         assert!(res.is_err());
//         match res {
//             Err(VaultError::EntryNotFound) => {}
//             _ => panic!("Expected EntryNotFound"),
//         }

//         cleanup_test_vault(name);
//     }

//     #[test]
//     fn test_delete_success() {
//         let name = "test_vault_delete_success";
//         let mut vault = initialize_vault(name.to_string()).unwrap();
//         let entry = Entry::new(
//             "test_entry".to_string(),
//             Some("testuser".to_string()),
//             Some("testpass123".to_string()),
//             Some("https://example.com".to_string()),
//             Some("test notes".to_string()),
//         );
//         vault.add_entry(entry).unwrap();
//         vault.save().unwrap();
//         let mut opt = Some(vault);
//         match handle_command_delete(&mut opt, "test_entry".to_string()) {
//             Ok(()) => assert!(true),
//             Err(e) => panic!("Expected successful deletion, got error: {}", e),
//         }
//         cleanup_test_vault(name);
//     } 


//     // Testing format of custom VaultErrors
//     #[test]
//     fn test_vault_error_display_strings() {
//         assert_eq!(format!("{}", VaultError::NameExists), "NAME ALREADY EXISTS");
//         assert_eq!(format!("{}", VaultError::NoVaultOpen), "NO VAULT IS OPEN");
//     }
// }
