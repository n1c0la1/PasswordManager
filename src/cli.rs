use crate::errors::*;
use crate::vault_entry_manager::*;
use crate::session::*;
use crate::vault_file_manager::initialize_vault;

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
        =============================================
                "#
        ;
        clear_terminal();

        println!("{frames}");
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

pub fn handle_command_init(option_name: Option<String>) -> Result<(), VaultError> {
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

    let key: SecretString = 'define_mw: loop {
        io::stdout().flush().unwrap();

        let password: SecretString = rpassword::prompt_password("Master-Password: ")?.into();

        if password.expose_secret().is_empty() {
            println!("The Master-Password may not be empty! Try again.");
            println!();
            continue 'define_mw;
        } else if password.expose_secret().len() < 3 {
            println!("The Password is too short! (minimum length is 3) Try again.");
            println!();
            continue 'define_mw;
        }

        let password_confirm: SecretString = rpassword::prompt_password("Please confirm the Master-Password: ")?.into();

        if password.expose_secret() != password_confirm.expose_secret() {
            println!("The passwords do not match, please try again.");
            println!();
            continue 'define_mw;
        }

        break 'define_mw password;
    };

    
    let spinner = spinner();
    spinner.enable_steady_tick(Duration::from_millis(80));
    println!();
    spinner.set_message(" Creating vault...");
    // let _ = create_new_vault(vault_name.clone(), key);
    match create_new_vault(vault_name.clone(), key) {
        Ok(()) => {
            spinner.finish_and_clear();
            println!("Vault '{}' created successfully! \n", vault_name);
            println!("Hint: Use 'open {}' to open it for the first time!", vault_name);

            Ok(())
        }
        Err(e) => {
            spinner.finish_and_clear();
            Err(e)
        }
    }
    
}

pub fn handle_command_add(
    option_vault: &mut Option<Vault>, 
    name: Option<String>, 
    username: Option<String>, 
    url: Option<String>, 
    notes: Option<String>, 
    password: Option<String>) -> Result<(), VaultError> {
    if let Some(vault) = option_vault {
        
        // Entry Name (REQUIRED)
        
        // Collect all existing entrynames
        let existing_names: Vec<String> = vault.get_entries().iter()
        .map(|e| e.get_entry_name().clone())
        .collect();
    
        let final_name = if let Some(n) = name {
            if existing_names.contains(&n) {
                return Err(VaultError::NameExists);
            } else {
                n
            }
        } else {
            let input_name;
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
                    print!("Generate password for entry (y/n): ");
                    let mut input_choice_gen = String::new();
                    io::stdin().read_line(&mut input_choice_gen)?;

                    if input_choice_gen.trim().eq_ignore_ascii_case("y") {
                        let length: i32;
                        let no_symbols: bool;

                        'input_length: loop {
                            print!("Enter desired password-length: ");
                            io::stdout().flush().unwrap();
                            let mut length_input = String::new();
                            io::stdin().read_line(&mut length_input)?;
                            if length_input.parse::<i32>().is_ok() {
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
                    None
                } else {
                    println!();
                    Some(loop_pw)
                }
            };

            let entry = Entry::new(final_name.clone(), final_username, final_pw, final_url, final_notes);

            let spinner = spinner();
            spinner.enable_steady_tick(Duration::from_millis(80));
            spinner.set_message("Adding PasswordEntry...");

            match vault.add_entry(entry) {
                Ok(_) => {
                    spinner.finish_and_clear();

                    println!();
                    println!("Entry '{}' added successfully!", final_name);
                    println!("Vault saved.\n");


                    Ok(())
                },
                Err(e) => {
                    spinner.finish_and_clear();
                    Err(e)
                },
            }
    } else {
        Err(VaultError::NoVaultOpen)
    }
}

pub fn handle_command_get(
    option_session: &mut Option<Session>, 
    option_vault: &mut Option<Vault>, 
    entry_name: String, 
    show: bool)
    -> Result<(), SessionError> 
    {
    if let Some(session) = option_session {

        // check Master-Password when show is passed.
        if show {
            let name_of_vault = option_vault.as_ref().unwrap().get_name();
            let master_input: SecretString = rpassword::prompt_password
                (format!("Enter master password for '{}': ", name_of_vault))?.into()
            ;
            session.verify_master_pw(master_input)?;
        }
        
        if let Some(vault) = option_vault {
            if let Some(entry) = vault.get_entry_by_name(&entry_name) {
                println!("\n==== Entry: {} ====", entry_name);
                println!("Username: {}", entry.get_user_name().as_deref().unwrap_or("--EMPTY--"));
                println!("URL:      {}", entry.get_url().as_deref().unwrap_or("--EMPTY--"));
                println!("Notes:    {}", entry.get_notes().as_deref().unwrap_or("--EMPTY--"));

                if show {
                    println!("Password: {}", entry.get_password().as_deref().unwrap_or("--EMPTY--"));
                } else {
                    println!("Password: *****");
                }
                println!();
                
                return Ok(());
            } else {
                println!("Error: Entry {} not found", entry_name);
                return Err(SessionError::VaultError(VaultError::EntryNotFound));
            }
        } else {
            return Err(SessionError::VaultError(VaultError::NoVaultOpen));
        }
    } 
    return Err(SessionError::SessionInactive);
}

pub fn handle_command_getall(
    option_session: &mut Option<Session>, 
    option_vault: &mut Option<Vault>, 
    show: bool)
    -> Result<(), SessionError> 
    {
    if let Some(session) = option_session {
        // check Master-Password when show is passed.
        if show {
            let name_of_vault = option_vault.as_ref().unwrap().get_name();
            let master_input: SecretString = rpassword::prompt_password
                (format!("Enter master password for '{}': ", name_of_vault))?.into()
            ;
            session.verify_master_pw(master_input)?;
        }

        if let Some(vault) = option_vault {
            let entries = vault.get_entries();

            if entries.is_empty() {
                return Err(SessionError::VaultError(VaultError::CouldNotGetEntry));
            }

            for entry in entries {
                println!("\n==== Entry: {} ====", entry.get_entry_name());
                println!("Username: {}", entry.get_user_name().as_deref().unwrap_or("--EMPTY--"));
                println!("URL:      {}", entry.get_url().as_deref().unwrap_or("--EMPTY--"));
                println!("Notes:    {}", entry.get_notes().as_deref().unwrap_or("--EMPTY--"));

                if show {
                    println!("Password: {}", entry.get_password().as_deref().unwrap_or("--EMPTY--"));
                } else {
                    println!("Password: *****");
                }
                println!();
            }
        }
        return Ok(());
    }
    return Err(SessionError::SessionInactive);
}

pub fn handle_command_delete(option_vault: &mut Option<Vault>, entry_to_delete: String) -> Result<(), SessionError> {
    if let Some(vault) = option_vault {
        let entry = vault.get_entry_by_name(&entry_to_delete)
            .ok_or(SessionError::VaultError(VaultError::EntryNotFound))?
        ;

        print!("Are you sure, you want to delete '{}'? (y/n): ", entry.get_entry_name());
        stdout().flush().unwrap();

        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;
                    
        if confirm.trim().eq_ignore_ascii_case("y") {
            // Master PW query maybe? TODO
            let spinner = spinner();
            spinner.enable_steady_tick(Duration::from_millis(80));
            spinner.set_message("Removing entry ...");
                        
            vault.remove_entry_by_name(&entry_to_delete);
                        
            spinner.finish_and_clear();
            
            println!();
            println!("Entry '{}' successfully removed!", entry_to_delete);
            return Ok(());
        } else {
            println!("Cancelled.\n");
            return Ok(());
        }   
    }
    return Err(SessionError::SessionInactive);
}

pub fn handle_command_deletevault(option_session: &mut Option<Session>) -> Result<(), SessionError> {
    println!();

    //deleting vault only acceptible, if a vault is currently open (-> session is active)
    if active_session(option_session){
        let session = option_session.as_mut().unwrap();
        let vault_name = session.vault_name.clone();
        println!("WARNING: You are about to PERMANENTLY delete vault '{}'!", vault_name);
        print!("Do you wish to continue? (y/n): ");

        stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().eq_ignore_ascii_case("n") {
            return Err(SessionError::VaultError(VaultError::AnyhowError(anyhow!("Cancelled."))));
        }

        println!();
        print!("Enter master password for {}: ", vault_name);
        stdout().flush().unwrap();
        let master_input: SecretString = rpassword::read_password()?.into();

        session.verify_master_pw(master_input)?;

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
                
            let expected = format!("DELETE {}", vault_name);
            let expected_low_case = format!("delete {}", vault_name);
            if trimmed == expected_low_case {
                println!();
                println!("You have to use capital letters! Try again or type exit.");
                continue 'input;
            } else if trimmed == expected {
                break 'input;
            } else if trimmed == "exit" {
                return Err(SessionError::VaultError(VaultError::AnyhowError(anyhow!("exit"))));
            } else {
                println!();
                println!("Wrong input! Try again or type exit.");
                continue 'input;
            }
        }

        let spinner = spinner();
        spinner.set_message(format!("Permanently deleting '{}' ...", vault_name));
        spinner.enable_steady_tick(Duration::from_millis(80));
        let path = Path::new("vaults").join(format!("{vault_name}.psdb"));

        session.end_session()?;

        fs::remove_file(path)?;
        spinner.finish_and_clear();
        println!();
        println!("Vault '{}' deleted permanently.", vault_name);

        Ok(())
    }
    else {
        return Err(SessionError::VaultError(VaultError::AnyhowError(anyhow!
            ("Due to RustPass' logic, you have to open the vault you want to delete first!")
        )))
    }  
}

pub fn handle_command_generate(length: i32, no_symbols: bool) -> Result<String, VaultError> {
    use rand::Rng;

    // Validierung der Länge
    if length <= 0 || length > 200 {
        return Err(VaultError::InvalidLength);
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

    copy_to_clipboard(&password)?;
    
    Ok(password)
}   

pub fn handle_command_change_master(option_session: &mut Option<Session>) -> Result<(), SessionError> {
    if let Some(session) = option_session {
        let session_vault_name = session.vault_name.clone();

        io::stdout().flush().unwrap();
        let old_password: SecretString = rpassword::prompt_password
            (format!("Enter the current master password for '{}': ", session_vault_name))?.into()
        ;
        session.verify_master_pw(old_password)?;
        
        let new_password: SecretString = 'input_new_master: loop {
            io::stdout().flush().unwrap();
            let input: SecretString = rpassword::prompt_password
                (format!("Enter the new master password for '{}': ", session_vault_name))?.into()
            ;
            
           if input.expose_secret().is_empty() {
                println!("New password cannot be empty!");
                continue 'input_new_master;
            }
            
            io::stdout().flush().unwrap();
            let confirm_new_passwd: SecretString = rpassword::prompt_password
                ("Confirm the new master password")?.into()
            ;
            
            if input.expose_secret() == confirm_new_passwd.expose_secret() {
                println!("Passwords do not match! Try again.");
                continue 'input_new_master;
            }
            //new_password = input;
            break 'input_new_master input;
        };

        session.change_master_pw(new_password)?;
        println!("Master password successfully updated!");

        let spinner = spinner();
        spinner.set_message("Automatically encrypting vault with new password ...");
        spinner.enable_steady_tick(Duration::from_millis(80));

        session.end_session()?;

        println!("All done!");
        println!("Hint: Use open <{}> to reopen your changed vault!", session_vault_name);
        println!();
        spinner.finish_and_clear();

        return Ok(());
    }

    Err(SessionError::SessionInactive)
}

pub fn handle_command_edit(option_vault: &mut Option<Vault>, entry_name: String) -> Result<(), VaultError> {

    let vault = option_vault.as_mut().ok_or(VaultError::NoVaultOpen)?;

    if !vault.entryname_exists(&entry_name) {
        return Err(VaultError::EntryNotFound);
    }

    // collecting current data, to avoid borrow checker
    let current_entry = vault.get_entries().iter()
        .find(|e| *e.get_entry_name() == entry_name)
        .ok_or(VaultError::EntryNotFound)?;
    
    let current_username = current_entry.get_user_name().clone();
    let current_url = current_entry.get_url().clone();
    let current_notes = current_entry.get_notes().clone();
    let current_password = current_entry.get_password().clone();
    let has_password = current_password.is_some();

    // Collect all existing entrynames except the own one
    let existing_names: Vec<String> = vault.get_entries().iter()
        .filter_map(|e| {
            if *e.get_entry_name() != entry_name {
                Some(e.get_entry_name().clone())
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
    //let vault = option_vault.as_mut().unwrap();
    let entry = vault.get_entry_by_name(&entry_name).ok_or(VaultError::EntryNotFound)?;

    if let Some(new_name) = new_entryname {entry.entryname = new_name;}
    if let Some(username) = new_username { entry.set_username(username); }
    if let Some(url) = new_url { entry.set_url(url); }
    if let Some(notes) = new_notes { entry.set_notes(notes); }
    if let Some(password) = new_password { entry.set_password(password); }

    // Get final name for printing
    let final_entry_name = entry.get_entry_name().clone();

    // Saving vault
    let spinner = spinner();
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner.set_message("Saving changes...");
    
    spinner.finish_and_clear();
    println!();
    println!("Entry '{}' updated successfully!", final_entry_name);
    println!("Vault saved.\n");

    Ok(())
}

pub fn handle_command_open(
    vault_to_open: String, 
    current_session: &mut Option<Session>, 
) 
-> Result<Session, SessionError> {
    // Check if vault file exists
    let path = Path::new("vaults").join(format!("{vault_to_open}.psdb"));
    if !path.exists() {
        return Err(SessionError::VaultError(VaultError::VaultDoesNotExist));
    }

    //check if the same vault is already open
    if let Some(session) = current_session.as_ref() {
        if vault_to_open == session.vault_name && active_session(current_session){
            println!();
            return Err(SessionError::VaultError(VaultError::AnyhowError(anyhow!("Vault '{}' already opened!", vault_to_open))));
        }

    }

    //close any existing session
    if let Some(session) = current_session.as_mut(){
        let old_name = session.vault_name.clone();

        println!();

        let spinner = spinner();
        spinner.set_message(format!("Closing currently opened vault '{}' first", old_name));
        spinner.enable_steady_tick(Duration::from_millis(80));

        match session.end_session() {
            Ok(()) => {
                spinner.finish_and_clear();
                println!("Vault '{}' closed successfully.", old_name);
            },
            Err(_) => {
                spinner.finish_and_clear();
                return Err(SessionError::VaultError(VaultError::CouldNotClose));
            }
        }
        *current_session = None;
    }
    
    println!();
    println!("Selected vault: {}", vault_to_open);
    
    io::stdout().flush().unwrap();
    let master: SecretString = rpassword::prompt_password("Enter master password: ")?.into();
    
    // Spinner
    let spinner = spinner();
    spinner.set_message("Opening vault ...");
    spinner.enable_steady_tick(Duration::from_millis(80));
    
    //starting session for new vault
    let mut new_session = Session::new(vault_to_open.clone());
    match new_session.start_session(master) {
        Ok(())               => {
            spinner.finish_and_clear();

            let opened_vault = new_session.opened_vault.as_mut().ok_or(SessionError::VaultError(VaultError::CouldNotOpen))?;

            println!();
            println!("╔═══════════════════════════════════════════╗");
            println!("║  Vault Opened Successfully                ║");
            println!("╠═══════════════════════════════════════════╣");
            println!("║  Vault: {}{}", 
                vault_to_open,
                " ".repeat(35 - vault_to_open.len().min(35))
            );
            println!("║  Entries: {}{}", 
                //new_session.opened_vault.as_ref().entries.len(),
                opened_vault.entries.len(),
                " ".repeat(33 - opened_vault.entries.len().to_string().len())
            );
            println!("║                                           ║");
            println!("║  Auto-close after 5 min inactivity        ║");
            println!("╚═══════════════════════════════════════════╝");
            println!();

            return Ok(new_session);
        }
        Err(SessionError::VaultError(VaultError::InvalidKey)) => {
            spinner.finish_and_clear();
            println!();
            return Err(SessionError::VaultError(VaultError::InvalidKey));
        }
        Err(SessionError::SessionActive) => {
            spinner.finish_and_clear();
            println!();
            println!("A session is already active!");

            // because main.rs prints the returned error and only VaultErrors can be returned
            // a session error has to be printed out here and the main function prints an empty string
            return Err(SessionError::VaultError(VaultError::AnyhowError(anyhow!(""))));
        }
        Err(e) => {
            return Err(SessionError::VaultError(VaultError::AnyhowError(anyhow!("Session error: {}", e))));
        }
    }
}

pub fn handle_command_close(option_session: &mut Option<Session>, force: bool) -> Result<LoopCommand, SessionError> {
    if let Some(session) = option_session {
        let open_vault_name = session.vault_name.clone();

        let spinner = spinner();
        spinner.set_message("Closing current vault and session ...");
        
        if force {
            spinner.enable_steady_tick(Duration::from_millis(80));
            
            session.end_session()?;
            
            spinner.finish_and_clear();
            println!("Successfully closed '{}'!", open_vault_name);
            
            return Ok(LoopCommand::Continue);
        }

        print!("Do you really want to close the current session and vault? (y/n): ");
        stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input);

        if input.trim().eq_ignore_ascii_case("n") {
            println!();
            println!("Cancelled.");
            return Ok(LoopCommand::Cancel)
        }

        spinner.enable_steady_tick(Duration::from_millis(80));
        
        session.end_session()?;

        spinner.finish_and_clear();
        println!("Successfully closed '{}'!", open_vault_name);

        return Ok(LoopCommand::Continue);
    }
    return Err(SessionError::SessionInactive);
} 

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

pub fn copy_to_clipboard(content: &str) -> Result<(), VaultError> {
    let mut clipboard = Clipboard::new().map_err(|_| VaultError::ClipboardError)?;
    clipboard.set_text(content.to_string()).map_err(|_| VaultError::ClipboardError)?;
    println!("Passwort wurde in die Zwischenablage kopiert!");
    Ok(())
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
