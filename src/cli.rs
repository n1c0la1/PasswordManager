use crate::vault_manager::*;
use crate::errors::*;
use password_manager::*;

use anyhow::anyhow;
use clap::{Parser, Subcommand, command};
use indicatif::{self, ProgressBar, ProgressStyle};
use rpassword;
use secrecy::ExposeSecret;
use secrecy::SecretString;
use std::env::join_paths;
use std::fmt::format;
use std::fs;
use std::io::stdout;
use std::io::{self, Read, Write};
use std::path::Path;
use std::{path::PathBuf, time::Duration};

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

    /// Clears terminal window.
    Clear {},

    /// Quits the input loop.
    Quit {
        //forces quit, normally "Do you really want to quit RustPass?"
        #[arg(short = 'f', long = "force")]
        force: bool,
    },
}

pub fn spinner() -> ProgressBar {
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
        io::stdin().read_line(&mut input)?;
        print!("{input}");
        input.trim().to_string()
    };

    let path = Path::new("vaults").join(format!("{vault_name}.psdb"));
    if path.exists() {
        return Err(VaultError::NameExists);
    }   

    //Define MasterPassword
    println!(
        "\nDefine the Master-Password for {}:",
        vault_name
    );

    let key = 'define_mw: loop {
        io::stdout().flush().unwrap();

        let password = rpassword::prompt_password("Master-Password: ")?;

        if password.is_empty() {
            println!("The Master-Password may not be empty! Try again.");
            println!();
            continue 'define_mw;
        } else if password.len() < 3 {
            println!("The Password is too short! (minimum length is 3) Try again.");
            println!();
            continue 'define_mw;
        }

        let password_confirm = rpassword::prompt_password("Please confirm the Master-Password: ")?;

        if password != password_confirm {
            println!("The passwords do not match, please try again.");
            println!();
            continue 'define_mw;
        }

        break 'define_mw password;
    };

    let vault = initialize_vault(vault_name.clone(), key)?;

    let spinner = spinner();
    spinner.enable_steady_tick(Duration::from_millis(80));
    println!();
    spinner.set_message(" Creating vault...");
    vault.save()?;
    spinner.finish_and_clear();

    println!("Vault '{}' created successfully! \n", vault_name);

    Ok(vault)
}

pub fn handle_command_add(
    option_vault: &mut Option<Vault>, 
    name: Option<String>, 
    username: Option<String>, 
    url: Option<String>, 
    notes: Option<String>, 
    password: Option<String>) -> Result<(), VaultError>{
    if let Some(vault) = option_vault {
        
        // Entry Name (REQUIRED)
        
        // Collect all existing entrynames
        let existing_names: Vec<String> = vault.entries.iter()
        .map(|e| e.entryname.clone())
        .collect()
        ;
    
    let final_name = if let Some(n) = name {
        if existing_names.contains(&n) {
            return Err(VaultError::NameExists);
        } else {
            n
        }
    } else {
        let mut input_name = String::new();
        'input: loop {
            println!("\n=== Adding new entry ===");
            println!("Entry name (required): ");
                print!("> ");
                io::stdout().flush().unwrap();
    
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                let trimmed_name: String = input.trim().to_string();
                if trimmed_name.is_empty() {
                    println!("Error: Entry name cannot be empty!");
                    continue 'input;
                } else if trimmed_name.eq("-EXIT-") {
                    return Ok(());
                }

                if existing_names.contains(&trimmed_name) {
                    println!("Error: the name '{}' already exists! Try again or type '-EXIT-'.", trimmed_name);
                    continue 'input;
                }

                input_name = trimmed_name;
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
            io::stdin().read_line(&mut input_username)?;
            let trimmed_username = input_username.trim().to_string();
            if trimmed_username.is_empty() {
                None
            } else {
                println!();
                Some(trimmed_username)
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
            io::stdin().read_line(&mut input_url)?;
            let trimmed_url = input_url.trim().to_string();
            if trimmed_url.is_empty() {
                None
            } else {
                println!();
                Some(trimmed_url)
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
            io::stdin().read_line(&mut input_url)?;
            let trimmed_notes = input_url.trim().to_string();
            if trimmed_notes.is_empty() {
                None
            } else {
                println!();
                Some(trimmed_notes)
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

                vault.save()?;
                println!();
                println!("Entry '{}' added successfully!", final_name);
                println!("Vault saved.\n");

                spinner.finish_and_clear();

                Ok(())
            },
            Err(e) => {
                Err(e)
            },
        }
    } else {
        Err(VaultError::NoVaultOpen)
    }
}

pub fn handle_command_get(option_vault: &mut Option<Vault>, entry_name: String, show: bool) -> Result<(), VaultError> {
    if option_vault.is_none() {
        return Err(VaultError::NoVaultOpen);
    }
    
    // check Master-Password when show is passed.
    if show {
        match master_pw_check(option_vault) {
            Ok(()) => {/* Do nothing */},
            Err(VaultError::AnyhowError(ref g)) if g.to_string() == "Exit" => {
                return Err(VaultError::AnyhowError(anyhow!("Exit")));
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    
    if let Some(vault) = option_vault {
        if let Ok(entry) = vault.get_entry_by_name(&entry_name) {
            println!("\n==== Entry: {} ====", entry_name);
            println!("Username: {}", entry.username.as_deref().unwrap_or("--EMPTY--"));
            println!("URL:      {}", entry.url.as_deref().unwrap_or("--EMPTY--"));
            println!("Notes:    {}", entry.notes.as_deref().unwrap_or("--EMPTY--"));

            if show {
                println!("Password: {}", entry.password.as_deref().unwrap_or("--EMPTY--"));
            } else {
                println!("Password: *****");
            }
            println!();
            
            Ok(())
        } else {
            println!("Entry {} not found", entry_name);
            Err(VaultError::CouldNotGetEntry)
        }
    } else {
        Err(VaultError::NoVaultOpen)
    }
}

pub fn handle_command_getall(option_vault: &mut Option<Vault>, show: bool) -> Result<(), VaultError> {
    if option_vault.is_none() {
        println!("No vault is active!");
        println!("Hint: Use init or open <vault-name>!");
    }

    // check Master-Password when show is passed.
    if show {
        match master_pw_check(&*option_vault) {
            Ok(())             => {/* Do nothing */},
            Err(VaultError::AnyhowError(ref g)) if g.to_string() == "Exit" => {
                return Err(VaultError::AnyhowError(anyhow!("Exit")));
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    if let Some(vault) = option_vault {
        let entries = &vault.entries;
        if entries.is_empty() {
            return Err(VaultError::CouldNotGetEntry);
        } else {
            for entry in entries {
                println!("\n==== Entry: {} ====", entry.entryname);
                println!("Username: {}", entry.username.as_deref().unwrap_or("--EMPTY--"));
                println!("URL:      {}", entry.url.as_deref().unwrap_or("--EMPTY--"));
                println!("Notes:    {}", entry.notes.as_deref().unwrap_or("--EMPTY--"));

                if show {
                    println!("Password: {}", entry.password.as_deref().unwrap_or("--EMPTY--"));
                } else {
                    println!("Password: *****");
                }
                println!();
            }
        }

        Ok(())
    } else {
        Err(VaultError::NoVaultOpen)
    }
}

pub fn handle_command_delete(option_vault: &mut Option<Vault>, entry_to_delete: String) -> Result<(), VaultError> {
    if let Some(vault) = option_vault {
        match vault.get_entry_by_name(&entry_to_delete) {
            Ok(entry) => {
                print!("Are you sure, you want to delete '{}'? (y/n): ", entry.entryname);
                stdout().flush().unwrap();

                let mut confirm = String::new();
                io::stdin().read_line(&mut confirm)?;
                
                if confirm.trim().eq_ignore_ascii_case("y") {
                    // Master PW query maybe? TODO
                    let spinner = spinner();
                    spinner.enable_steady_tick(Duration::from_millis(80));
                    spinner.set_message("Removing entry ...");
                    
                    vault.remove_entry_by_name(entry_to_delete.clone())?;
                    vault.save()?;
                    
                    spinner.finish_and_clear();
                    println!("Entry '{}' successfully removed!", entry_to_delete);
                    println!("  Vault saved.\n");
                } else {
                    println!("Cancelled.\n");
                }
            },
            Err(VaultError::CouldNotGetEntry) => {
                return Err(VaultError::CouldNotGetEntry);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Err(VaultError::NoVaultOpen)
}

pub fn handle_command_deletevault(option_vault: &mut Option<Vault>) -> Result<(), VaultError> {
    println!();

    if let Some(vault) = option_vault {
        let vault_name = vault.get_name();
        println!("WARNING: You are about to PERMANENTLY delete vault '{}'!", vault_name);
        print!("Do you wish to continue? (y/n): ");

        stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            return Err(VaultError::AnyhowError(anyhow!("Cancelled.")));
        }

        println!();
        print!("Enter master password for {}: ", vault_name);
        stdout().flush().unwrap();
        let master_input: SecretString = SecretString::new(rpassword::read_password()?.into());

        if !master_input.expose_secret().eq(vault.key.as_ref().unwrap().as_str()) {
            return Err(VaultError::AnyhowError(anyhow!("Invalid master password! Deletion cancelled.")));
        }

        println!();
        println!("Password verified.");
        println!();

        println!("FINAL WARNING: This action CANNOT be undone!");

        'input: loop {
            println!("Type 'DELETE {}' to confirm: ", vault_name);
    
            let mut input = String::new();
            stdout().flush().unwrap();
            io::stdin().read_line(&mut input)?;
            let trimmed = input.trim();
            
            let expected = format!("DELETE {}", vault.get_name());
            let expected_low_case = format!("delete {}", vault.get_name());
            if trimmed == expected_low_case {
                println!();
                println!("You have to use capital letters! Try again or type exit.");
                continue 'input;
            } else if trimmed == expected {
                break 'input;
            } else if trimmed == "exit" {
                return Err(VaultError::AnyhowError(anyhow!("exit")));
            } else {
                println!();
                println!("Wrong input! Try again or type exit.");
                continue 'input;
            }
        }

        let spinner = spinner();
        spinner.enable_steady_tick(Duration::from_millis(80));
        let path = Path::new("vaults").join(format!("{vault_name}.psdb"));

        fs::remove_file(path)?;
        spinner.finish_and_clear();
        println!();
        println!("Vault '{}' deleted permanently.", vault_name);

        return Ok(());
    }

    Err(VaultError::AnyhowError(anyhow!("Due to RustPass' logic, you have to open the vault you want to delete first!")))
}

pub fn handle_command_generate(length: i32, no_symbols: bool) -> Result<String, VaultError> {
    use rand::Rng;

    // Validierung der Länge
    if length <= 0 {
        return Err(VaultError::AnyhowError(anyhow!("Password length must be gerater than 0!")));
    }

    // Zeichensatz basierend auf no_symbols Flag
    let charset = if no_symbols {
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
    } else {
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}|;:,.<>?"
    };

    let chars: Vec<char> = charset.chars().collect();
    // Generiere Passwort durch zufällige Auswahl aus charset
    let password: String = (0..length)
        .map(|_| {
            let idx = rand::rng().random_range(0..charset.len());
            chars[idx]
        })
        .collect();

    println!("\n┌─────────────────────────────────────────┐");
    println!("│ Generated Password                      │");
    println!("├─────────────────────────────────────────┤");
    println!("│ {:<40} ", 
        password);
    println!("├─────────────────────────────────────────┤");
    println!("│ Length: {} characters{}                 ", 
        length, " ".repeat(27 - length.to_string().len()));
    println!("│ Symbols: {}{}                           ", 
        if no_symbols { "No" } else { "Yes" }, if no_symbols { " " } else { "" }.repeat(33));
    println!("└─────────────────────────────────────────┘\n");

    /* Password copy to clipboard? TODO mit neuer flag -c copy & Abfrage
    use arboard::Clipboard;
    let mut clipboard = Clipboard::new().expect("Clipboard nicht verfügbar");
    clipboard
        .set_text(password.clone())
        .expect("Konnte nicht in Zwischenablage kopieren");

    println!("Passwort wurde in die Zwischenablage kopiert!");
        */
    Ok(password)
}   

pub fn handle_command_change_master(option_vault: &mut Option<Vault>) -> Result<(), VaultError> {
    if let Some(vault) = option_vault {
        print!("Enter current master password: ");
        io::stdout().flush().unwrap();
        let old_password = rpassword::read_password().unwrap();
        
        let mut new_password = String::new();

        'input_new_master: loop {
            print!("Enter new master password: ");
            io::stdout().flush().unwrap();
            let input = rpassword::read_password().unwrap();
            
            if input.is_empty() {
                println!("New password cannot be empty!");
                continue 'input_new_master;
            }
            
            print!("Confirm new master password: ");
            io::stdout().flush().unwrap();
            let confirm_password = rpassword::read_password().unwrap();
            
            if input != confirm_password {
                println!("Passwords do not match!");
                continue 'input_new_master;
            }

            new_password = input;
            break 'input_new_master;
        }

        match vault.change_master_key(old_password, new_password) {
            Ok(_) => {
                let spinner = spinner();
                spinner.enable_steady_tick(Duration::from_millis(80));
                spinner.set_message("Re-encrypting vault...");
                
                vault.save()?;
                
                spinner.finish_and_clear();
                println!("Master password changed successfully!");
                println!("Vault re-encrypted with new password.\n");

                return Ok(());
            }
            Err(e) => println!("Error: {}\n", e),
        }
    }

    Err(VaultError::NoVaultOpen)
}

pub fn handle_command_edit(option_vault: &mut Option<Vault>, entry_name: String) -> Result<(), VaultError> {

    let vault = match option_vault {
        Some(v) => v,
        None => return Err(VaultError::NoVaultOpen),
    };

    if !vault.entryname_exists(&entry_name) {
        return Err(VaultError::EntryNotFound);
    }

    // collecting current data, to avoid borrow checker
    let current_entry = vault.entries.iter()
        .find(|e| e.entryname == entry_name)
        .ok_or(VaultError::EntryNotFound)?;
    
    let current_username = current_entry.username.clone();
    let current_url = current_entry.url.clone();
    let current_notes = current_entry.notes.clone();
    let current_password = current_entry.password.clone();
    let has_password = current_password.is_some();

    // Collect all existing entrynames except the own one
    let existing_names: Vec<String> = vault.entries.iter()
        .filter_map(|e| {
            if e.entryname != entry_name {
                Some(e.entryname.clone())
            } else {
                None
            }
        })
        .collect();

    // Collecting user inputs
    println!("\n==== Editing entry: '{}' ====", entry_name);
    println!("Hint: (Press enter to keep current value)\n");

    // Entryname
    let new_entryname = loop {
        print!("New entry name [current: {}]: ", entry_name);
        stdout().flush().unwrap();
        
        let mut input_entryname = String::new();
        io::stdin().read_line(&mut input_entryname)?;
        let trimmed_input = input_entryname.trim().to_string();

        if trimmed_input.is_empty() || trimmed_input == entry_name {
            // Keep current name
            break None;
        } else if existing_names.contains(&trimmed_input) {
            println!("Error: An entry with the name '{}' already exists! Try a different name.", trimmed_input);
            continue;
        } else {
            break Some(trimmed_input);
        }
    };

    // Username 
    print!("New username [current: {}]: ", current_username.as_deref().unwrap_or("--EMPTY--"));
    stdout().flush().unwrap();
    let mut input_username = String::new();
    io::stdin().read_line(&mut input_username)?;
    let new_username = if input_username.trim().is_empty() {
        None
    } else {
        Some(input_username.trim().to_string())
    };

    // URL 
    print!("New URL [current: {}]: ", current_url.as_deref().unwrap_or("--EMPTY--"));
    stdout().flush().unwrap();
    let mut input_url = String::new();
    io::stdin().read_line(&mut input_url)?;
    let new_url = if input_url.trim().is_empty() {
        None
    } else {
        Some(input_url.trim().to_string())
    };

    // Notes sammeln
    print!("New notes [current: {}]: ", current_notes.as_deref().unwrap_or("--EMPTY--"));
    stdout().flush().unwrap();
    let mut input_notes = String::new();
    io::stdin().read_line(&mut input_notes)?;
    let new_notes = if input_notes.trim().is_empty() {
        None
    } else {
        Some(input_notes.trim().to_string())
    };

    // Password 
    let new_password = if has_password {
        'input_new_pw: loop {
        print!("New password (press Enter to keep current, type 'clear' to remove): ");
        stdout().flush().unwrap();
        let input_password = rpassword::read_password()?;
        
        if input_password.is_empty() {
            // Keep current password
            break 'input_new_pw None;
        } else if input_password == "clear" {
            break 'input_new_pw Some("".to_string());
        } else {
            // Confirm new password
            print!("Confirm new password: ");
            stdout().flush().unwrap();
            let confirm = rpassword::read_password()?;
            
            if input_password != confirm {
                println!("Passwords do not match! Try again.");
                continue 'input_new_pw;
            } else {
                break 'input_new_pw Some(input_password);
            }
        }}   
    } else {
        // No password exists, ask if user wants to add one
        print!("Add password? (y/n): ");
        stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().eq_ignore_ascii_case("y") {
            let mut loop_pw = String::new();
            'input_pw: loop {
                print!("Enter password: ");
                stdout().flush().unwrap();
                let input_password = rpassword::read_password()?;
                
                if input_password.is_empty() {
                    break 'input_pw;
                }

                print!("Confirm password: ");
                stdout().flush().unwrap();
                let confirm = rpassword::read_password()?;
                
                if input_password != confirm {
                    println!("Passwords do not match! Try again:");
                    println!();
                    continue 'input_pw;
                }

                loop_pw = input_password;
                break 'input_pw;
            }
            if loop_pw.is_empty() {
                None
            } else {
                Some(loop_pw)
            }
        } else {
            None
        }
    };

    // Write changes to Entry
    let vault = option_vault.as_mut().unwrap();
    let entry = vault.get_entry_by_name(&entry_name)?;

    if let Some(new_name) = new_entryname {entry.entryname = new_name;}
    if let Some(username) = new_username { entry.set_username(username); }
    if let Some(url) = new_url { entry.set_url(url); }
    if let Some(notes) = new_notes { entry.set_notes(notes); }
    if let Some(password) = new_password { entry.set_password(password); }

    // Get final name for printing
    let final_entry_name = entry.entryname.clone();

    // Saving vault
    let spinner = spinner();
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner.set_message("Saving changes...");
    
    vault.save()?;
    
    spinner.finish_and_clear();
    println!();
    println!("Entry '{}' updated successfully!", final_entry_name);
    println!("Vault saved.\n");

    Ok(())
}

pub fn handle_command_open(option_vault: &mut Option<Vault>, vault_to_open: String) -> Result<Vault, VaultError> {
    let path = Path::new("vaults").join(format!("{vault_to_open}.psdb"));
    if !path.exists() {
        return Err(VaultError::VaultDoesNotExist);
    }


    // Closing old vault
    if let Some(vault) = option_vault.take() {
        if vault_to_open.eq(vault.get_name()) {
            println!();
            println!("This vault is already open!");
            // Nothing to handle here, just restart with Ok
            return Ok(vault);
        } else {
            let old_name = vault.name.clone();
            
            let spinner = spinner();
            spinner.set_message(format!("Closing vault {} ...", old_name));
            spinner.enable_steady_tick(Duration::from_millis(80));

            vault.close()?;

            spinner.finish_and_clear();
            println!("Vault {} successfully closed.", old_name);
        }
    }

    println!();
    // Opening new one
    print!("Enter master-password for {}: ", vault_to_open);
    io::stdout().flush().unwrap();
    let input_pw = rpassword::read_password()?;

    let spinner = spinner();
    spinner.set_message("Opening vault ...");
    spinner.enable_steady_tick(Duration::from_millis(80));

    match open_vault(&vault_to_open, input_pw) {
        Ok(opened_vault) => {
            spinner.finish_and_clear();
            println!();
            println!("Vault {} successfully opened!", vault_to_open);
            Ok(opened_vault)
        },
        Err(e) => {
            Err(e)
        }
    }
}

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

pub fn handle_command_quit(force: bool) -> Result<LoopCommand, VaultError> {
    if force {
        println!("Quitting RustPass...");
        return Ok(LoopCommand::Break);
    } 

    print!("Are you sure you want to quit? (y/n): ");
    io::stdout().flush().unwrap();


    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        println!("Quitting RustPass...");
        // Closing the vault is happening in main.rs to avoid cloning.
        Ok(LoopCommand::Break)
    } else {
        println!("Cancelled. \n");
        io::stdout().flush().unwrap();
        Ok(LoopCommand::Continue)
    }
}
