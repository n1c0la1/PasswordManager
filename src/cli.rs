use crate::vault_manager::*;

use clap::{Parser, Subcommand, command};
use indicatif::{self, ProgressBar, ProgressStyle};
use password_manager::{clear_terminal, intro_animation};
use rpassword;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::{path::PathBuf, time::Duration};
use std::thread::{self};

#[derive(Parser)]
#[command(name = "pw")]
pub struct CLI {
    #[command(subcommand)]
    pub command: CommandCLI,
}

pub enum LoopCommand {
    Continue,
    Break,
}

#[derive(Subcommand)]
pub enum CommandCLI {
    /// Initializes a new PasswordManager.
    Init {
        #[arg(short = 'n', long)]
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
        show: bool,
        // Maybe an Option to directly copy the password to clipboard without showing it.
    },

    /// List all Entrys.
    // maybe implement filters e.g. all passwords with that email, or on that URL.
    ShowEntries {
        // Name of vault.
        vault: String,

        // Show passwords or not.
        show: bool,
    },

    /// Generate a password.
    // maybe implement interaction (abfrage) if with special cases, numbers etc.
    Generate {
        length: i32,

        no_symbols: bool,
    },

    /// Remove an entry from Database.
    Delete {
        name: String,
    },

    /// Change the Masterpassword.
    // implement not visible, old password required. Verschlüsselt Vault sofort
    ChangeMaster {},

    Vaults {},

    /// Modify a given password
    //
    Modify {
        name: String,
    },

    /// Opens given vault.
    Open {
        name: String,
    },

    /// Switches to given vault, while closing the current one.
    Switch {
        name: String,
    },

    /// Clears terminal window.
    Clear {},

    /// Quits the input loop
    Quit {
        //forces quit, normally "Do you really want to quit RustPass?"
        #[arg(short = 'f', long)]
        force: bool,
    },
}

fn spinner() -> ProgressBar {
    let spinner: ProgressBar = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["|", "/", "-", "\\"])
            .template("{spinner} {msg}")
            .unwrap(),
    );
    spinner
}

pub fn handle_command_init(option_name: Option<String>) -> Result<Vault, VaultError> {
    println!("\nInitializing new vault: ");

    let vault_name = if let Some(name) = option_name {
        name
    } else {
        println!("What should be the name of your new vault?");
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        print!("{input}");
        input.trim().to_string()
    };

    let path = format!("vaults/{vault_name}.psdb");
    if Path::new(&path).exists() {
        return Err(VaultError::NameExists);
    }   

    //Define MasterPassword
    let key = 'define_mw: loop {
        println!(
            "\n---------------\nDefine the Master-Password for {}:",
            vault_name
        );
        io::stdout().flush().unwrap();

        let password = rpassword::prompt_password("Password: ").unwrap();

        let password_confirm = rpassword::prompt_password("Please confirm the password: ").unwrap();

        if password != password_confirm {
            println!("The passwords do not match, please try again.");
            continue 'define_mw;
        }

        break 'define_mw password;
    };

    let vault = initialize_vault(vault_name.clone(), key)?;

    let spinner = spinner();
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner.set_message(" Creating vault...");
    vault.save();
    spinner.finish_and_clear();

    println!("\nVault '{}' created successfully! \n", vault_name);

    Ok(vault)
}

pub fn handle_command_add(option_vault: &mut Option<Vault>, name: Option<String>, username: Option<String>, url: Option<String>, notes: Option<String>, password: Option<String>) {
    if let Some(vault) = option_vault {
        println!("\n=== Adding new entry ===");
        
        // Entry Name (REQUIRED)
        let final_name = if let Some(n) = name {
            n
        } else {
            let mut input_name = String::new();
            'input: loop {
                println!("Entry name (required): ");
                print!("> ");
                io::stdout().flush().unwrap();
    
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Error reading your input.");
    
                let trimmed: String = input.trim().to_string();
                if trimmed.is_empty() {
                    println!("Error: Entry name cannot be empty!");
                    continue 'input;
                }
                input_name = trimmed;
                break 'input;
            }
            input_name
        };

        println!("\n(Press Enter to skip optional fields)");
        
        // Username
        let final_username = if let Some(u) = username {
            Some(u)
        } else {
            println!("Username: ");
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input_username = String::new();
            io::stdin().read_line(&mut input_username).expect("Error reading your input.");
            if input_username.is_empty() {
                None
            } else {
                println!();
                Some(input_username)
            }
        };

        // URL
        let final_url = if let Some(url) = url {
            Some(url)
        } else {
            println!("URL: ");
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input_url = String::new();
            io::stdin().read_line(&mut input_url).expect("Error reading your input.");
            if input_url.is_empty() {
                None
            } else {
                println!();
                Some(input_url)
            }
        };

        // Notes
        let final_notes = if let Some(n) = notes {
            Some(n)
        } else {
            println!("Type additional notes, if needed (enter submits it): ");
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input_url = String::new();
            io::stdin().read_line(&mut input_url).expect("Error reading your input.");
            if input_url.is_empty() {
                None
            } else {
                println!();
                Some(input_url)
            }
        };

        // Password
        let final_pw = if let Some(p) = password {
            Some(p)
        } else {
            let mut loop_pw = String::new();
            'input_pw: loop {
                print!("Enter password for the entry (or press Enter to skip): ");
                io::stdout().flush().unwrap();
                let input_password = rpassword::read_password().unwrap();
                
                print!("Please confirm the password: ");
                io::stdout().flush().unwrap();
                let confirm = rpassword::read_password().unwrap();
                
                if input_password != confirm {
                    println!("The passwords do not match! Try again:");
                    println!();
                    continue 'input_pw;
                }

                loop_pw = input_password;
                break 'input_pw;
            }
            if loop_pw.is_empty() {
                None
            } else {
                println!();
                Some(loop_pw)
            }
        };

        let entry = Entry::new(final_name.clone(), final_username, final_pw, final_url, final_notes);

        match vault.add_entry(entry) {
            Ok(_) => {
                let spinner = spinner();
                spinner.enable_steady_tick(Duration::from_millis(80));
                spinner.set_message("Adding PasswordEntry...");

                vault.save();
                println!("Entry '{}' added successfully!", final_name);
                println!("Vault saved.\n");

                spinner.finish_and_clear();
            }
            Err(e) => println!("Error: {}", e),
        }
    } else {
        println!("!! No vault is open to add an entry to !!");
        println!("Consider using open <vault-name> or init!");
    }
}

pub fn handle_command_get() {}
pub fn handle_command_show_entries() {}
pub fn handle_command_delete() {}
pub fn handle_command_generate() {}
pub fn handle_command_change_master() {}
pub fn handle_command_modify() {}
pub fn handle_command_open() {}
pub fn handle_command_switch() {}
pub fn handle_command_vaults(current_vault: &Option<Vault>) {
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
                
                let current_vault_name = current_vault.as_ref()
                    .map(|v| v.get_name());
                
                for vault_name in vault_files {
                    if Some(&vault_name.as_str()) == current_vault_name.as_ref() {
                        println!("  → {} (currently open)", vault_name);
                    } else {
                        println!("    {}", vault_name);
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

pub fn handle_command_quit(option_vault: Option<Vault>, force: bool) -> LoopCommand {
    if force {
        println!("Quitting RustPass...");
        return LoopCommand::Break;
    } 

    print!("Are you sure you want to quit? (y/n): ");
    io::stdout().flush().unwrap();


    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Error reading your input.");

    if input.trim().eq_ignore_ascii_case("y") {
        println!("Quitting RustPass...");
        if let Some(vault) = option_vault {
            vault.close();
        }
        LoopCommand::Break
    } else {
        println!("Cancelled. \n");
        io::stdout().flush().unwrap();
        LoopCommand::Continue
    }
}
