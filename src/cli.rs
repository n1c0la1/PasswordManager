use crate::vault_manager::*;

use clap::{Parser, Subcommand, command};
use indicatif::{self, ProgressBar, ProgressStyle};
use rpassword;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::{path::PathBuf, time::Duration};

#[derive(Parser)]
#[command(name = "pw")]
pub struct CLI {
    #[command(subcommand)]
    pub command: CommandCLI,
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
        name: String,

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

    Open {
        name: String,
    },

    Switch {
        name: String,
    },

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
        print!("What should be the name of your new vault? \n> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        print!("{input}");
        input.trim().to_string()
    };

    let key = String::new();

    //Define MasterPassword
    'define_mw: loop {
        println!(
            "\n---------------\nDefine the Master-Password for {}:",
            vault_name
        );
        io::stdout().flush().unwrap();

        let password = rpassword::prompt_password("Password: ").unwrap();
        println!("{password}");

        let password_confirm = rpassword::prompt_password("Please confirm the password: ").unwrap();
        println!("{password_confirm}");

        if password != password_confirm {
            println!("The passwords do not match, please try again.");
            continue 'define_mw;
        }

        break 'define_mw;
    }

    let vault = initialize_vault(vault_name.clone(), key)?;

    let spinner = spinner();
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner.set_message(" Creating vault...");
    vault.save();
    spinner.finish_and_clear();

    println!("\nVault '{}' created successfully! \n", vault_name);
    // ToDo!: insert at main/CommandCLI::Init
    /*match handle_command_init(name) {
        Ok(vault) => current_vault = Some(vault),
        Err(e) => println!("Error: {}", e)
    }*/
    Ok(vault)
}

pub fn handle_command_add(option_vault: &mut Option<Vault>, name: String, username: Option<String>, url: Option<String>, notes: Option<String>, password: Option<String>) {
    /*if !check_vaults_exist() {
                    eprintln!("There are currently no vaults, consider using 'init' to create one!");
                    continue 'interactive_shell;
                }

                // Auto-open vault if not open
                if !ensure_vault_open(&mut current_vault) {
                    continue;
                }*/
    
    if let Some(vault) = option_vault {
        // hätte ich weggelassen, da dass password davor oder danach hinzugefügt werden kann
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
                let spinner = spinner();
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
pub fn handle_command_quit(option_vault: Option<Vault>, force: bool) {
    //loop must be replaced by while-loop
    /*if force {
        println!("Quitting RustPass...");
        break;
    } 

    print!("Are you sure you want to quit? (y/n): ");
    io::stdout().flush().unwrap();


    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    if input.trim().eq_ignore_ascii_case("y") {
        println!("Quitting RustPass...");
        if let Some(vault) = option_vault {
            vault.close();
        }
        break 'interactive_shell;
    } else {
        println!("Cancelled. \n");
        io::stdout().flush().unwrap();
    }*/
}