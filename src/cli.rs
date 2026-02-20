use crate::errors::*;
use crate::session::*;
use crate::vault_entry_manager::*;
use crate::vault_file_manager::{list_vaults, vault_exists};
//use crate::vault_file_manager::initialize_vault;
//use crate::test::*;

use anyhow::anyhow;
use arboard::Clipboard;
use clap::{Parser, Subcommand, command};
use indicatif::{self, ProgressBar, ProgressStyle};
use passgenr::charsets;
use passgenr::random_password;
use rpassword;
use secrecy::ExposeSecret;
use secrecy::SecretString;
use std::io::stdout;
use std::io::{self, Write};
use std::time::Duration;
use zxcvbn::zxcvbn;

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

        // w as in website
        #[arg(short = 'w', long)]
        url: Option<String>,

        #[arg(short, long)]
        password: Option<String>,

        #[arg(short, long)]
        notes: Option<String>,
    },

    /// Get an Entry of the Database.
    // Accepts entry name or URL
    Get {
        //name of the Element to be shown, or a URL to search for
        name: String,

        // Specifies whether the password should be shown in the command line.
        #[arg(short = 's', long)]
        show: bool,

        // Copy credentials to clipboard instead of displaying
        #[arg(short = 'c', long)]
        copy: bool,
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
        length: u32,

        #[arg(short = 'f', long = "no_symbols")]
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

        #[arg(short = 't', long = "timeout")]
        timeout: Option<u64>,
        // check timeout von dem Mutex in main erwartet u64
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

static CANCEL_ARG: &'static str = "--CANCEL";

pub fn clear_terminal() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

pub fn intro_animation() {
    let frames = r#"
=== RustPass ================================
Secure • Fast • Rust-Powered Password Manager
=============================================
        "#;
    clear_terminal();

    println!("{frames}");
}

pub fn spinner() -> ProgressBar {
    let spinner: ProgressBar = ProgressBar::new_spinner();
    let style = ProgressStyle::default_spinner()
        .tick_strings(&["|", "/", "-", "\\"])
        .template("{spinner} {msg}")
        .unwrap_or_else(|_| ProgressStyle::default_spinner());
    spinner.set_style(style);
    spinner
}

pub fn handle_command_init(option_name: Option<String>) -> Result<(), VaultError> {
    println!("\nInitializing new vault: ");

    let vault_name: String = 'define_vault_name: loop {
        if let Some(option_vault_name) = option_name.clone() {
            match check_vault_name(&option_vault_name) {
                Err(_) => {
                    println!("Invalid name.");
                    println!("Suggestion: Vault name should be less than 64 characters long");
                    println!(
                        "Suggestion: Vault name is allowed to contain only letters, numbers, \"_\" and \"-\" \n"
                    );
                    continue 'define_vault_name;
                }
                Ok(()) => break 'define_vault_name option_vault_name,
            }
        } else {
            println!("What should be the name of your new vault?");
            print!("> ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            print!("{input}");
            let input = input.trim().to_string();

            if input.eq(CANCEL_ARG) {
                return Err(VaultError::ActionCancelled);
            }

            match check_vault_name(&input) {
                Err(_) => {
                    println!("Invalid name.");
                    println!("Suggestion: Vault name should be less than 64 characters long");
                    println!(
                        "Suggestion: Vault name is allowed to contain only letters, numbers, \"_\" and \"-\" \n"
                    );
                    continue 'define_vault_name;
                }
                Ok(()) => break 'define_vault_name input,
            }
        };
    };

    if crate::vault_file_manager::vault_exists(&vault_name)? {
        return Err(VaultError::NameExists);
    }

    //Define MasterPassword
    println!("\nDefine the Master-Password for {}:", vault_name);

    let key: SecretString = 'define_mw: loop {
        io::stdout().flush().unwrap();

        let password: SecretString = rpassword::prompt_password("Master-Password: ")?.into();

        if password.expose_secret() == CANCEL_ARG {
            return Err(VaultError::ActionCancelled);
        }

        match check_password_strength(&password) {
            Err(_) => continue 'define_mw,
            Ok(()) => {
                let password_confirm: SecretString =
                    rpassword::prompt_password("Please confirm the Master-Password: ")?.into();

                if password.expose_secret() != password_confirm.expose_secret() {
                    println!("The passwords do not match, please try again.");
                    println!();
                    continue 'define_mw;
                }

                break 'define_mw password;
            }
        }
    };

    let spinner = spinner();
    spinner.enable_steady_tick(Duration::from_millis(80));
    println!();
    spinner.set_message(" Creating vault...");
    match create_new_vault(vault_name.clone(), key) {
        Ok(()) => {
            spinner.finish_and_clear();
            println!("Vault '{}' created successfully! \n", vault_name);
            println!(
                "Hint: Use 'open {}' to open it for the first time!",
                vault_name
            );

            Ok(())
        }
        Err(e) => {
            spinner.finish_and_clear();
            Err(e)
        }
    }
}

pub fn handle_command_add(
    option_session: &mut Option<Session>,
    name: Option<String>,
    username: Option<String>,
    url: Option<String>,
    notes: Option<String>,
    password: Option<String>,
) -> Result<(), SessionError> {
    let session = option_session
        .as_mut()
        .ok_or(SessionError::SessionInactive)?;

    let vault = session
        .opened_vault
        .as_mut()
        .ok_or(SessionError::VaultError(VaultError::NoVaultOpen))?;

    // Entry Name (REQUIRED)

    // Collect all existing entrynames
    let existing_names: Vec<String> = vault
        .get_entries()
        .iter()
        .map(|e| e.get_entry_name().clone())
        .collect();

    let final_name = if let Some(n) = name {
        if existing_names.contains(&n) {
            return Err(SessionError::VaultError(VaultError::NameExists));
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
            } else if trimmed_name.eq(CANCEL_ARG) {
                return Err(SessionError::VaultError(VaultError::ActionCancelled));
            }

            if existing_names.contains(&trimmed_name) {
                println!(
                    "Error: the name '{}' already exists! Try again or type '{}'.",
                    trimmed_name, CANCEL_ARG
                );
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
        add_password_to_entry()?
    };

    let entry = Entry::new(
        final_name.clone(),
        final_username,
        final_pw,
        final_url,
        final_notes,
    );

    let spinner = spinner();
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner.set_message("Adding PasswordEntry...");

    match vault.add_entry(entry) {
        Ok(_) => {
            spinner.finish_and_clear();

            println!();
            println!("Entry '{}' added successfully!", final_name);
            println!();

            Ok(())
        }
        Err(e) => {
            spinner.finish_and_clear();
            Err(SessionError::VaultError(e))
        }
    }
}

pub fn handle_command_get(
    option_session: &mut Option<Session>,
    entry_name_or_url: String,
    show: bool,
    copy: bool,
) -> Result<(), SessionError> {
    let session = option_session
        .as_mut()
        .ok_or(SessionError::SessionInactive)?;

    // check Master-Password when show is passed.
    if show {
        let name_of_vault: &String = match &session.opened_vault {
            Some(vault) => vault.get_name(),
            None => {
                return Err(SessionError::VaultError(VaultError::NoVaultOpen));
            }
        };
        let master_input: SecretString =
            rpassword::prompt_password(format!("Enter master password for '{}': ", name_of_vault))?
                .into();
        session.verify_master_pw(master_input)?;
    }

    let vault = session
        .opened_vault
        .as_mut()
        .ok_or(SessionError::VaultError(VaultError::NoVaultOpen))?;

    // First, try to find by exact entry name
    let entry_opt = vault.get_entry_by_name(&entry_name_or_url);

    let entry = if let Some(e) = entry_opt {
        // Found by name
        e
    } else {
        // Not found by name, try URL-based lookup
        let matches: Vec<&crate::vault_entry_manager::Entry> = vault
            .entries
            .iter()
            .filter(|entry| {
                if let Some(entry_url) = entry.get_url() {
                    url_matches(entry_url, &entry_name_or_url)
                } else {
                    false
                }
            })
            .collect();

        match matches.len() {
            0 => {
                println!(
                    "Entry '{}' not found (tried name and URL lookup)",
                    entry_name_or_url
                );
                return Err(SessionError::VaultError(VaultError::EntryNotFound));
            }
            1 => matches[0],
            _ => {
                // Multiple matches found
                println!("\nMultiple entries found for '{}':", entry_name_or_url);
                for (i, e) in matches.iter().enumerate() {
                    println!(
                        "  {}. {} ({})",
                        i + 1,
                        e.get_entry_name(),
                        e.get_url().as_deref().unwrap_or("no URL")
                    );
                }
                println!("\nPlease use the exact entry name to select one.");
                return Ok(());
            }
        }
    };

    // Handle --copy flag
    if copy {
        use arboard::Clipboard;
        let username = entry.get_user_name().as_deref().unwrap_or("");
        let password = entry.get_password().as_deref().unwrap_or("");

        if username.is_empty() && password.is_empty() {
            println!("Entry has no username or password to copy");
            return Ok(());
        }

        // Format: username\npassword
        let clipboard_content = format!("{}\n{}", username, password);

        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_text(clipboard_content) {
                Ok(_) => {
                    let duration = 30;
                    println!(
                        "✓ Credentials copied to clipboard for '{}'",
                        entry.get_entry_name()
                    );
                    println!("  (Clipboard will be cleared in {} seconds)", &duration);

                    clear_clipboard_after(duration);
                }
                Err(e) => {
                    println!("Failed to copy to clipboard: {}", e);
                }
            },
            Err(e) => {
                println!("Failed to access clipboard: {}", e);
            }
        }

        return Ok(());
    }

    // Display entry (normal behavior)
    println!("\n==== Entry: {} ====", entry.get_entry_name());
    println!(
        "Username: {}",
        entry.get_user_name().as_deref().unwrap_or("--EMPTY--")
    );
    println!(
        "URL:      {}",
        entry.get_url().as_deref().unwrap_or("--EMPTY--")
    );
    println!(
        "Notes:    {}",
        entry.get_notes().as_deref().unwrap_or("--EMPTY--")
    );

    if show {
        println!(
            "Password: {}",
            entry.get_password().as_deref().unwrap_or("--EMPTY--")
        );
    } else {
        println!("Password: *****");
    }
    println!();

    Ok(())
}

pub fn handle_command_getall(
    option_session: &mut Option<Session>,
    show: bool,
) -> Result<(), SessionError> {
    let session = option_session
        .as_mut()
        .ok_or(SessionError::SessionInactive)?;

    // check Master-Password when show is passed.
    if show {
        let name_of_vault: &String = match &session.opened_vault {
            Some(vault) => vault.get_name(),
            None => {
                return Err(SessionError::VaultError(VaultError::NoVaultOpen));
            }
        };
        let master_input: SecretString =
            rpassword::prompt_password(format!("Enter master password for '{}': ", name_of_vault))?
                .into();
        session.verify_master_pw(master_input)?;
    }

    let vault = session
        .opened_vault
        .as_mut()
        .ok_or(SessionError::VaultError(VaultError::NoVaultOpen))?;

    let entries = vault.get_entries();

    if entries.is_empty() {
        return Err(SessionError::VaultError(VaultError::CouldNotGetEntry));
    }

    for entry in entries {
        println!("\n==== Entry: {} ====", entry.get_entry_name());
        println!(
            "Username: {}",
            entry.get_user_name().as_deref().unwrap_or("--EMPTY--")
        );
        println!(
            "URL:      {}",
            entry.get_url().as_deref().unwrap_or("--EMPTY--")
        );
        println!(
            "Notes:    {}",
            entry.get_notes().as_deref().unwrap_or("--EMPTY--")
        );

        if show {
            println!(
                "Password: {}",
                entry.get_password().as_deref().unwrap_or("--EMPTY--")
            );
        } else {
            println!("Password: *****");
        }
        println!();
    }

    Ok(())
}

pub fn handle_command_delete(
    option_session: &mut Option<Session>,
    entry_to_delete: String,
) -> Result<(), SessionError> {
    let session = option_session
        .as_mut()
        .ok_or(SessionError::SessionInactive)?;
    let vault = session
        .opened_vault
        .as_mut()
        .ok_or(SessionError::VaultError(VaultError::NoVaultOpen))?;
    let entry = if let Some(entry) = vault.get_entry_by_name(&entry_to_delete) {
        entry
    } else {
        println!();
        println!(
            "'{}' not found in current vault \"{}\"!",
            entry_to_delete,
            vault.get_name()
        );
        return Err(SessionError::VaultError(VaultError::EntryNotFound));
    };

    print!(
        "Are you sure, you want to delete '{}'? (y/n): ",
        entry.get_entry_name()
    );
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
    } else {
        println!("Cancelled.\n");
    }
    Ok(())
}

pub fn handle_command_deletevault(
    option_session: &mut Option<Session>,
) -> Result<(), SessionError> {
    println!();

    //deleting vault only acceptible, if a vault is currently open (-> session is active)
    if !active_session(option_session) {
        return Err(SessionError::VaultError(VaultError::AnyhowError(anyhow!(
            "Due to RustPass' logic, you have to open the vault you want to delete first!"
        ))));
    }

    let session = option_session
        .as_mut()
        .ok_or(SessionError::SessionInactive)?;
    let vault_name = session.vault_name.clone();
    println!(
        "WARNING: You are about to PERMANENTLY delete vault '{}'!",
        vault_name
    );
    print!("Do you wish to continue? (y/n): ");

    stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("n") {
        return Err(SessionError::VaultError(VaultError::ActionCancelled));
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
            println!(
                "You have to use capital letters! Try again or type '{}'.",
                CANCEL_ARG
            );
            continue 'input;
        } else if trimmed == expected {
            break 'input;
        } else if trimmed == CANCEL_ARG {
            return Err(SessionError::VaultError(VaultError::ActionCancelled));
        } else {
            println!();
            println!("Wrong input! Try again or type '{}'.", CANCEL_ARG);
            continue 'input;
        }
    }

    let spinner = spinner();
    spinner.set_message(format!("Permanently deleting '{}' ...", vault_name));
    spinner.enable_steady_tick(Duration::from_millis(80));
    session.end_session()?;
    crate::vault_file_manager::delete_vault_file(&vault_name).map_err(SessionError::VaultError)?;
    spinner.finish_and_clear();
    println!();
    println!("Vault '{}' deleted permanently.", vault_name);

    Ok(())
}

pub fn handle_command_generate(length: u32, no_symbols: bool) -> Result<String, SessionError> {
    // Validierung der Länge
    if length <= 1 || length > 200 {
        return Err(SessionError::VaultError(VaultError::InvalidLength));
    }

    // Zeichensatz basierend auf no_symbols Flag
    let charset = if no_symbols {
        charsets::ALPHANUMERIC
    } else {
        charsets::ASCII
    };

    let password = random_password(charset, length as usize, "")?;

    println!("\n┌─────────────────────────────────────────┐");
    println!("│ Generated Password                      │");
    println!("├─────────────────────────────────────────┤");
    println!("│ {:<40} ", password);
    println!("├─────────────────────────────────────────┤");
    println!(
        "│ Length: {} characters{}                 ",
        length,
        " ".repeat(27 - length.to_string().len())
    );
    println!(
        "│ Symbols: {}{}                           ",
        if no_symbols { "No" } else { "Yes" },
        if no_symbols { " " } else { "" }.repeat(33)
    );
    println!("└─────────────────────────────────────────┘\n");

    copy_to_clipboard(&password)?;
    Ok(password)
}

pub fn handle_command_change_master(
    option_session: &mut Option<Session>,
) -> Result<(), SessionError> {
    if !active_session(option_session) {
        return Err(SessionError::SessionInactive);
    }
    let session = option_session
        .as_mut()
        .ok_or(SessionError::SessionInactive)?;
    let session_vault_name = session.vault_name.clone();

    io::stdout().flush().unwrap();
    let old_password: SecretString = rpassword::prompt_password(format!(
        "Enter the current master password for '{}': ",
        session_vault_name
    ))?
    .into();
    session.verify_master_pw(old_password)?;

    let new_password: SecretString = 'input_new_master: loop {
        io::stdout().flush().unwrap();
        let input: SecretString = rpassword::prompt_password(format!(
            "Enter the new master password for '{}': ",
            session_vault_name
        ))?
        .into();

        if input.expose_secret() == CANCEL_ARG {
            return Err(SessionError::VaultError(VaultError::ActionCancelled));
        }

        match check_password_strength(&input) {
            Err(_) => continue 'input_new_master,
            Ok(()) => {
                io::stdout().flush().unwrap();
                let confirm_new_passwd: SecretString =
                    rpassword::prompt_password("Confirm the new master password")?.into();

                if input.expose_secret() != confirm_new_passwd.expose_secret() {
                    println!("Passwords do not match! Try again.");
                    continue 'input_new_master;
                }
                //new_password = input;
                break 'input_new_master input;
            }
        }
    };

    session.change_master_pw(new_password)?;
    println!("Master password successfully updated!");

    let spinner = spinner();
    spinner.set_message("Automatically encrypting vault with new password ...");
    spinner.enable_steady_tick(Duration::from_millis(80));

    session.end_session()?;

    println!("All done!");
    println!(
        "Hint: Use open <{}> to reopen your changed vault!",
        session_vault_name
    );
    println!();
    spinner.finish_and_clear();

    Ok(())
}

pub fn handle_command_edit(
    option_session: &mut Option<Session>,
    entry_name: String,
) -> Result<(), SessionError> {
    let session = option_session
        .as_mut()
        .ok_or(SessionError::SessionInactive)?;

    let vault = session
        .opened_vault
        .as_mut()
        .ok_or(SessionError::VaultError(VaultError::NoVaultOpen))?;

    if !vault.entryname_exists(&entry_name) {
        return Err(SessionError::VaultError(VaultError::EntryNotFound));
    }

    // collecting current data, to avoid borrow checker
    let current_entry = vault
        .get_entries()
        .iter()
        .find(|e| *e.get_entry_name() == entry_name)
        .ok_or(SessionError::VaultError(VaultError::EntryNotFound))?;

    let current_username = current_entry.get_user_name().clone();
    let current_url = current_entry.get_url().clone();
    let current_notes = current_entry.get_notes().clone();
    let current_password = current_entry.get_password().clone();
    let has_password = current_password.is_some();

    // Collect all existing entrynames except the own one
    let existing_names: Vec<String> = vault
        .get_entries()
        .iter()
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
            println!(
                "Error: An entry with the name '{}' already exists! Try a different name.",
                trimmed_input
            );
            continue;
        } else {
            break Some(trimmed_input);
        }
    };

    // Username
    print!(
        "New username [current: {}]: ",
        current_username.as_deref().unwrap_or("--EMPTY--")
    );
    stdout().flush().unwrap();
    let mut input_username = String::new();
    io::stdin().read_line(&mut input_username)?;
    let new_username = if input_username.trim().is_empty() {
        None
    } else {
        Some(input_username.trim().to_string())
    };

    // URL
    print!(
        "New URL [current: {}]: ",
        current_url.as_deref().unwrap_or("--EMPTY--")
    );
    stdout().flush().unwrap();
    let mut input_url = String::new();
    io::stdin().read_line(&mut input_url)?;
    let new_url = if input_url.trim().is_empty() {
        None
    } else {
        Some(input_url.trim().to_string())
    };

    // Notes sammeln
    print!(
        "New notes [current: {}]: ",
        current_notes.as_deref().unwrap_or("--EMPTY--")
    );
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
            }
        }
    } else {
        add_password_to_entry()?
    };

    // Write changes to Entry
    //let vault = option_vault.as_mut().unwrap();
    let entry = vault
        .get_entry_by_name(&entry_name)
        .ok_or(SessionError::VaultError(VaultError::EntryNotFound))?;

    if let Some(new_name) = new_entryname {
        entry.entryname = new_name;
    }
    if let Some(username) = new_username {
        entry.set_username(username);
    }
    if let Some(url) = new_url {
        entry.set_url(url);
    }
    if let Some(notes) = new_notes {
        entry.set_notes(notes);
    }
    if let Some(password) = new_password {
        entry.set_password(password);
    }

    // Get final name for printing
    let final_entry_name = entry.get_entry_name().clone();

    // Saving vault
    let spinner = spinner();
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner.set_message("Saving changes...");

    spinner.finish_and_clear();
    println!();
    println!("Entry '{}' updated successfully!", final_entry_name);

    Ok(())
}

pub fn handle_command_open(
    vault_to_open: String,
    current_session: &mut Option<Session>,
    timeout: &Option<u64>,
) -> Result<Session, SessionError> {
    // Check if vault file exists
    match vault_exists(&vault_to_open) {
        Ok(true) => { /* Do nothing */ }
        Ok(false) => return Err(SessionError::VaultError(VaultError::VaultDoesNotExist)),
        Err(e) => return Err(SessionError::VaultError(e)),
    }

    //check if the same vault is already open
    if let Some(session) = current_session.as_ref() {
        if vault_to_open == session.vault_name && active_session(current_session) {
            println!();
            return Err(SessionError::VaultError(VaultError::AnyhowError(anyhow!(
                "Vault '{}' already opened!",
                vault_to_open
            ))));
        }
    }

    //close any existing session
    if let Some(session) = current_session.as_mut() {
        let old_name = session.vault_name.clone();

        println!();

        let spinner = spinner();
        spinner.set_message(format!(
            "Closing currently opened vault '{}' first",
            old_name
        ));
        spinner.enable_steady_tick(Duration::from_millis(80));

        match session.end_session() {
            Ok(()) => {
                spinner.finish_and_clear();
                println!("Vault '{}' closed successfully.", old_name);
            }
            Err(SessionError::SessionInactive) => {
                spinner.finish_and_clear();
                // Session was already inactive, just verify it's cleared
            }
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
    if let Some(minutes) = timeout {
        new_session.wished_timeout = minutes * 60; // Convert minutes to seconds
    }
    match new_session.start_session(master) {
        Ok(()) => {
            spinner.finish_and_clear();

            let opened_vault = new_session
                .opened_vault
                .as_mut()
                .ok_or(SessionError::VaultError(VaultError::CouldNotOpen))?;

            println!();
            println!("╔═══════════════════════════════════════════╗");
            println!("║  Vault Opened Successfully                ║");
            println!("╠═══════════════════════════════════════════╣");
            let vault_line = format!("  Vault: {}", vault_to_open);
            println!("║{: <43}║", vault_line);
            // the String line gets filled up until it has at least 43 chars

            let entries_line = format!("  Entries: {}", opened_vault.entries.len());
            println!("║{: <43}║", entries_line);
            println!("║                                           ║");

            let timeout_minutes = timeout.unwrap_or(5);
            let timeout_line = format!("  Auto-close after {} min inactivity", timeout_minutes);
            println!("║{: <43}║", timeout_line);
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
            return Err(SessionError::VaultError(VaultError::AnyhowError(anyhow!(
                ""
            ))));
        }
        Err(e) => {
            return Err(SessionError::VaultError(VaultError::AnyhowError(anyhow!(
                "Session error: {}",
                e
            ))));
        }
    }
}

pub fn handle_command_close(
    option_session: &mut Option<Session>,
    force: bool,
) -> Result<LoopCommand, SessionError> {
    let session = option_session
        .as_mut()
        .ok_or(SessionError::SessionInactive)?;

    let open_vault_name = session.vault_name.clone();

    let spinner = spinner();

    if force {
        spinner.set_message("Closing current vault and session ...");
        spinner.enable_steady_tick(Duration::from_millis(80));

        session.end_session()?;

        spinner.finish_and_clear();
        println!("Successfully closed '{}'!", open_vault_name);

        return Ok(LoopCommand::Continue);
    }

    print!("Do you really want to close the current session and vault? (y/n): ");
    stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("n") {
        println!();
        println!("Cancelled.");
        return Ok(LoopCommand::Cancel);
    }

    spinner.set_message("Closing current vault and session ...");
    spinner.enable_steady_tick(Duration::from_millis(80));

    session.end_session()?;

    spinner.finish_and_clear();
    println!("Successfully closed '{}'!", open_vault_name);

    return Ok(LoopCommand::Continue);
}

pub fn handle_command_vaults(current_session: &Option<Session>) {
    println!("\n=== Available Vaults ===");

    match list_vaults() {
        Ok(mut vault_files) => {
            if vault_files.is_empty() {
                println!("  (no vaults found)");
                println!("\nCreate a new vault with: init <vault_name>");
            } else {
                vault_files.sort();

                if current_session.is_none() || !active_session(current_session) {
                    for vault_name in vault_files {
                        println!("    {}", vault_name);
                    }
                } else {
                    let current_vault_name = current_session
                        .as_ref()
                        .and_then(|s| s.opened_vault.as_ref())
                        .map(|v| v.get_name());

                    for vault_name in vault_files {
                        if Some(&vault_name) == current_vault_name {
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

fn copy_to_clipboard(content: &str) -> Result<(), SessionError> {
    let mut clipboard =
        Clipboard::new().map_err(|_| SessionError::VaultError(VaultError::ClipboardError))?;
    clipboard
        .set_text(content.to_string())
        .map_err(|_| SessionError::VaultError(VaultError::ClipboardError))?;
    println!("Password copied to clipboard!");

    let duration = 30;
    println!("  (Clipboard will be cleared in {} seconds)", &duration);

    clear_clipboard_after(duration);
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
            let length: u32;
            let no_symbols: bool;

            'input_length: loop {
                print!("Enter desired password-length: ");
                let mut length_input = String::new();
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut length_input)?;
                let trimmed_input = length_input.trim();

                if trimmed_input.eq(CANCEL_ARG) {
                    return Ok(None);
                }

                if let Ok(len) = trimmed_input.parse::<u32>() {
                    length = len;
                    break 'input_length;
                }
            }

            print!("Use symbols? (y/n): ");
            io::stdout().flush().unwrap();
            let mut no_symbols_input = String::new();
            io::stdin().read_line(&mut no_symbols_input)?;
            if no_symbols_input.trim().eq_ignore_ascii_case("y") {
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

fn check_password_strength(password: &SecretString) -> Result<(), VaultError> {
    if password.expose_secret().is_empty() {
        println!("The Master-Password may not be empty! Try again.");
        println!();
        return Err(VaultError::WeakPassword);
    }
    if password.expose_secret().len() < 10 {
        println!("The Password is too short! (minimum length is 10 characters) Try again.");
        println!();
        return Err(VaultError::WeakPassword);
    }

    let min_number_of_guesses = 1_000_000_000;
    let estimate = zxcvbn(&password.expose_secret(), &[])?;
    if estimate.guesses() < min_number_of_guesses {
        println!("Estimated guesses: {}", estimate.guesses());
        println!(
            "The password is too weak, do not use common passwords. Try combining unrelated words."
        );
        return Err(VaultError::WeakPassword);
    }
    //println!("Estimated guesses: {}", estimate.guesses());

    Ok(())
}

fn check_vault_name(vault_name: &str) -> Result<(), VaultError> {
    if vault_name.len() > 64 {
        return Err(VaultError::InvalidVaultName);
    }
    if vault_name.is_empty() {
        return Err(VaultError::InvalidVaultName);
    }
    if !vault_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(VaultError::InvalidVaultName);
    }

    Ok(())
}

pub fn url_matches(entry_url: &str, target_url: &str) -> bool {
    // Extract domain from URLs for matching
    // e.g., "https://github.com" matches "https://github.com/login"
    let entry_domain = extract_domain(entry_url)
        .trim_start_matches("www.")
        .to_string();
    let target_domain = extract_domain(target_url)
        .trim_start_matches("www.")
        .to_string();
    entry_domain == target_domain
}

pub fn extract_domain(url: &str) -> String {
    let url = url.trim();
    let url_with_scheme = if url.contains("://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    };

    if let Ok(parsed) = url::Url::parse(&url_with_scheme) {
        if let Some(host) = parsed.host_str() {
            let host = host.strip_prefix("www.").unwrap_or(host);
            return host.to_string();
        }
    }

    url.to_string()
}

pub fn clear_clipboard_after(duration: u64) {
    use arboard::Clipboard;
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(duration));
        if let Ok(mut clip) = Clipboard::new() {
            let _ = clip.set_text("".to_string());
        }
    });
}
// tests

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    // Helper-function to delete testvault-file
    fn cleanup_test_vault(vault_name: &str) {
        let path = format!("vaults/{}.psdb", vault_name);
        if Path::new(&path).exists() {
            fs::remove_file(&path).ok();
        }
    }

    fn create_test_session(name: &str) -> Session {
        let pw: SecretString = "pw".into();
        let _ = create_new_vault(name.into(), pw.clone());
        let mut session = Session::new(name.into());
        let _ = &session.start_session(pw);
        session
    }

    // ================== QUIT TESTS ==================

    #[test]
    fn test_handle_command_quit_with_force() {
        let result = handle_command_quit(true);
        assert!(result.is_ok());
        match result.unwrap() {
            LoopCommand::Continue => assert!(true),
            LoopCommand::Cancel => panic!("Expected Break, got Continue"),
        }
    }

    #[test]
    fn test_loop_command_variants() {
        let continue_cmd = LoopCommand::Continue;
        let break_cmd = LoopCommand::Cancel;
        match continue_cmd {
            LoopCommand::Continue => assert!(true),
            LoopCommand::Cancel => panic!("Wrong variant for Continue"),
        }
        match break_cmd {
            LoopCommand::Cancel => assert!(true),
            LoopCommand::Continue => panic!("Wrong variant for Break"),
        }
    }

    // ================== CLEAR TESTS ==================

    #[test]
    fn test_handle_command_clear() {
        // test whether no panic
        handle_command_clear();
    }
    // ================== EDIT TESTS ==================

    #[test]
    fn test_edit_without_vault() {
        let session = Session::new("name".into());
        let mut option_session: Option<Session> = Some(session);
        let result = handle_command_edit(&mut option_session, "test_entry".to_string());
        assert!(result.is_err());
        match result {
            Err(SessionError::VaultError(VaultError::NoVaultOpen)) => assert!(true),
            _ => panic!("Expected NoVaultOpen error"),
        }
    }

    #[test]
    fn test_edit_nonexistent_entry() {
        let vault_name = "test_vault_edit_nonexistent";
        let session = create_test_session(vault_name);
        let result = handle_command_edit(&mut Some(session), "nonexistent".to_string());
        assert!(result.is_err());
        match result {
            Err(SessionError::VaultError(VaultError::EntryNotFound)) => assert!(true),
            _ => panic!("Expected EntryNotFound error"),
        }
        cleanup_test_vault(vault_name);
    }

    #[test]
    fn test_edit_entry_exists() {
        let vault_name = "test_vault_edit_exists";
        let session = create_test_session(vault_name);
        let entry = Entry::new(
            "test_entry".to_string(),
            Some("testuser".to_string()),
            Some("testpass123".to_string()),
            Some("https://example.com".to_string()),
            Some("test notes".to_string()),
        );
        let mut vault = session.opened_vault.unwrap();
        vault.add_entry(entry).unwrap();
        assert!(vault.entryname_exists("test_entry"));
        cleanup_test_vault(vault_name);
    }

    #[test]
    fn test_edit_entry_verification() {
        let vault_name = "test_vault_edit_verification";
        let mut session = create_test_session(vault_name);
        let entry = Entry::new(
            "test_entry".to_string(),
            Some("original_user".to_string()),
            Some("original_pass".to_string()),
            Some("https://original.com".to_string()),
            Some("original notes".to_string()),
        );
        let vault = session.opened_vault.as_mut().unwrap();
        vault.add_entry(entry).unwrap();
        // close vault and open again
        session.save().unwrap();
        session.opened_vault = None;
        session.start_session("pw".into()).unwrap();
        let vault = session.opened_vault.as_mut().unwrap();
        // handle edit needs user input -> simulating what happens in handle
        let entry = vault.get_entry_by_name(&"test_entry".to_string()).unwrap();
        entry.set_username("modified_user".to_string());
        entry.set_url("https://modified.com".to_string());
        session.save().unwrap();
        session.opened_vault = None;
        session.start_session("pw".into()).unwrap();
        let mut vault = session.opened_vault.unwrap();
        // check
        let entry = vault.get_entry_by_name(&"test_entry".to_string()).unwrap();
        assert_eq!(*entry.get_user_name(), Some("modified_user".to_string()));
        assert_eq!(*entry.get_url(), Some("https://modified.com".to_string()));
        cleanup_test_vault(vault_name);
    }

    // ================== GETALL TESTS ==================
    #[test]
    fn test_handle_command_getall_no_vault() {
        let vault_name = "test_vault";
        let session = create_test_session(vault_name);
        let res = handle_command_getall(&mut Some(session), false);
        assert!(res.is_err());
        match res {
            Err(SessionError::VaultError(VaultError::CouldNotGetEntry)) => {}
            _ => panic!("Expected No Entry"),
        }
        cleanup_test_vault(vault_name);
    }

    #[test]
    fn test_getall_success() {
        let vault_name = "test_vault";
        let mut session = create_test_session(vault_name);
        let entry = Entry::new(
            "test_entry".to_string(),
            Some("testuser".to_string()),
            Some("testpass123".to_string()),
            Some("https://example.com".to_string()),
            Some("test notes".to_string()),
        );
        let vault = session.opened_vault.as_mut().unwrap();
        vault.add_entry(entry).unwrap();
        let res = handle_command_getall(&mut Some(session), false);
        assert!(res.is_ok());
        cleanup_test_vault(vault_name);
    }

    // ================== GENERATE TESTS ==================
    #[test]
    fn test_generate_invalid_length() {
        let res = handle_command_generate(0, true);
        assert!(res.is_err());
        match res {
            Err(SessionError::VaultError(VaultError::InvalidLength)) => {}
            _ => panic!("Expected InvalidLength for length 0"),
        }
    }

    #[test]
    fn test_generate_success() {
        let length = 12;
        let res = handle_command_generate(length, false);
        assert!(res.is_ok());
        let password = res.unwrap();
        assert_eq!(password.len(), length as usize);
    }

    #[test]
    fn test_generate_correct_characters() {
        let length = 20;
        let res = handle_command_generate(length, true);
        assert!(res.is_ok());
        let password = res.unwrap();
        assert_eq!(password.len(), length as usize);
        for c in password.chars() {
            assert!(
                c.is_ascii_alphanumeric(),
                "Password contains non-alphanumeric character: {}",
                c
            );
        }
    }

    // ================== DELETE TESTS ==================
    #[test]
    fn test_delete_entry_not_found() {
        let vault_name = "test_vault_delete_not_found";
        let session = create_test_session(vault_name);
        let res = handle_command_delete(&mut Some(session), "does_not_exist".to_string());
        assert!(res.is_err());
        match res {
            Err(SessionError::VaultError(VaultError::EntryNotFound)) => {}
            _ => panic!("Expected EntryNotFound"),
        }
        cleanup_test_vault(vault_name);
    }

    #[test]
    fn test_delete_success() {
        let vault_name = "test_vault_delete_success";
        let mut session = create_test_session(vault_name);
        let entry = Entry::new(
            "test_entry".to_string(),
            Some("testuser".to_string()),
            Some("testpass123".to_string()),
            Some("https://example.com".to_string()),
            Some("test notes".to_string()),
        );
        let vault = session.opened_vault.as_mut().unwrap();
        vault.add_entry(entry).unwrap();
        match handle_command_delete(&mut Some(session), "test_entry".to_string()) {
            Ok(()) => assert!(true),
            Err(e) => panic!("Expected successful deletion, got error: {}", e),
        }
        cleanup_test_vault(vault_name);
    }

    // ================== FORMAT OF CUSTOM VAULTERRORS TESTS ==================
    // Testing format of custom VaultErrors
    #[test]
    fn test_vault_error_display_strings() {
        assert_eq!(format!("{}", VaultError::NameExists), "NAME ALREADY EXISTS");
        assert_eq!(format!("{}", VaultError::NoVaultOpen), "NO VAULT IS OPEN");
    }

    // ================== ADD TESTS ==================

    //Test: create a new entry with everything to be added to the vault -> success
    #[test]
    fn test_add_entry() {
        let vault_name = "test_vault_add";

        let session = create_test_session(vault_name);
        let mut opt_session = Some(session);

        let entry_name = Some("test_entry".to_string());
        let username = Some("original_user".to_string());
        let url = Some("https://original.com".to_string());
        let notes = Some("original notes".to_string());
        let password = Some("original_password".to_string());

        let result =
            handle_command_add(&mut opt_session, entry_name, username, url, notes, password);

        assert!(result.is_ok());

        let vault_ref = opt_session.as_ref().unwrap().opened_vault.as_ref().unwrap();
        assert!(vault_ref.entryname_exists("test_entry"));

        cleanup_test_vault(vault_name.into());
    }

    #[test]
    fn test_add_entry_with_existing_entry() {
        let vault_name = "test_vault_add";

        let session = create_test_session(vault_name);
        let mut opt_session = Some(session);

        //let vault = &mut session.opened_vault;

        let entry_name = Some("test_entry".to_string());
        let username = Some("orisiginal_user".to_string());
        let url = Some("https://original.com".to_string());
        let notes = Some("original notes".to_string());
        let password = Some("original_password".to_string());

        let first_add = handle_command_add(
            &mut opt_session,
            entry_name.clone(),
            username.clone(),
            url.clone(),
            notes.clone(),
            password.clone(),
        );
        assert!(first_add.is_ok());

        let vault_ref = opt_session.as_ref().unwrap().opened_vault.as_ref().unwrap();
        assert!(vault_ref.entryname_exists("test_entry"));

        let second_add =
            handle_command_add(&mut opt_session, entry_name, username, url, notes, password);
        assert!(matches!(
            second_add,
            Err(SessionError::VaultError(VaultError::NameExists))
        ));

        cleanup_test_vault(vault_name.into());
    }

    // ================== GET TESTS ==================

    //Test: no session active -> error
    #[test]
    fn test_get_entry_no_session() {
        let mut opt_session = None;
        let result = handle_command_get(&mut opt_session, "unimportant".to_string(), false, false);
        assert!(matches!(result, Err(SessionError::SessionInactive)));
    }

    //Test: get an entry by name, do not show password -> success
    #[test]
    fn test_get_entry_dont_show_pw() {
        let vault_name = "test_get";

        let session = create_test_session(vault_name);
        let mut opt_session = Some(session);

        //Add test entry
        let entry_name = Some("test_entry".to_string());
        let username = Some("original_user".to_string());
        let url = Some("https://original.com".to_string());
        let notes = Some("original notes".to_string());
        let password = Some("original_password".to_string());

        let add_entry =
            handle_command_add(&mut opt_session, entry_name, username, url, notes, password);
        assert!(add_entry.is_ok());

        let vault_ref = opt_session.as_ref().unwrap().opened_vault.as_ref().unwrap();
        assert!(vault_ref.entryname_exists("test_entry"));

        let result = handle_command_get(&mut opt_session, "test_entry".to_string(), false, false);
        assert!(result.is_ok());

        cleanup_test_vault(vault_name);
    }

    //Test: get entry with non-existent name -> error
    #[test]
    fn test_get_entry_with_nonexistent_name() {
        let vault_name = "test_get";

        let session = create_test_session(vault_name);
        let mut opt_session = Some(session);

        let result = handle_command_get(&mut opt_session, "nonexistent".to_string(), false, false);
        assert!(matches!(
            result,
            Err(SessionError::VaultError(VaultError::EntryNotFound))
        ));

        cleanup_test_vault(vault_name.into());
    }

    // ================== PASSWORD STRENGTH TESTS ==================

    #[test]
    fn test_valid_password() {
        let password: SecretString = "rustSEPtest".into();
        let result = check_password_strength(&password);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_passsword() {
        let password: SecretString = "".into();
        let result = check_password_strength(&password);
        assert!(matches!(result, Err(VaultError::WeakPassword)));
    }

    #[test]
    fn test_short_password() {
        let password: SecretString = "too_short".into();
        let result = check_password_strength(&password);
        assert!(matches!(result, Err(VaultError::WeakPassword)));
    }

    #[test]
    fn test_weak_password() {
        let password: SecretString = "weakpassword".into();
        let result = check_password_strength(&password);
        assert!(matches!(result, Err(VaultError::WeakPassword)));
    }

    // ================== VAULT NAME TESTS ==================
    #[test]
    fn test_valid_vault_name() {
        let name = "valid_-Name1";
        let result = check_vault_name(name);
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_vault_name() {
        let name = "";
        let result = check_vault_name(name);
        assert!(matches!(result, Err(VaultError::InvalidVaultName)));
    }

    #[test]
    fn test_long_vault_name() {
        let name = "a".repeat(65);
        let result = check_vault_name(&name);
        assert!(matches!(result, Err(VaultError::InvalidVaultName)));
    }

    #[test]
    fn test_invalid_name() {
        let name = "/";
        let result = check_vault_name(name);
        assert!(matches!(result, Err(VaultError::InvalidVaultName)));

        let name2 = "\\";
        let result2 = check_vault_name(name2);
        assert!(matches!(result2, Err(VaultError::InvalidVaultName)));

        let name3 = ".";
        let result3 = check_vault_name(name3);
        assert!(matches!(result3, Err(VaultError::InvalidVaultName)));
    }

    // ================== EXTENSION TESTS ==================
    // Tests for the extension: URL comparision and domain extraction
    #[test]
    fn test_extract_domain_1() {
        assert_eq!(extract_domain("example.com"), "example.com");
    }

    #[test]
    fn test_extract_domain_2_login() {
        assert_eq!(
            extract_domain("https://www.example.com/login"),
            "example.com"
        );
    }

    #[test]
    fn test_extract_domain_3_port() {
        assert_eq!(
            extract_domain("https://www.example.com:8080"),
            "example.com"
        );
    }

    #[test]
    fn test_extract_domain_4_port_and_path() {
        assert_eq!(
            extract_domain("https://www.example.com:8080/path"),
            "example.com"
        );
    }

    #[test]
    fn test_extract_domain_5_invalid_url() {
        assert_eq!(extract_domain("invalid_url"), "invalid_url");
    }

    #[test]
    fn test_extract_domain_6() {
        assert_eq!(
            extract_domain("https://subdomain.example.com"),
            "subdomain.example.com"
        );
    }

    #[test]
    fn test_url_comparision() {
        assert!(url_matches("www.example.com", "example.com"));
    }

    #[test]
    fn test_url_comparison_2_login() {
        assert!(url_matches("www.github.com/login", "github.com"));
    }

    #[test]
    fn test_url_comparison_3_http() {
        assert!(url_matches("http://github.com", "github.com"));
    }

    #[test]
    fn test_url_comparison_4_different() {
        assert!(!url_matches("https://www.example.com", "different.com"));
    }

    #[test]
    fn test_url_comparison_5_subdomain() {
        assert!(!url_matches("mail.example.com", "example.com"));
    }

    #[test]
    fn test_extract_and_compare() {
        assert!(url_matches(
            &extract_domain("https://www.example.com/login"),
            &extract_domain("example.com")
        ));
        assert!(url_matches(
            &extract_domain("http://github.com"),
            &extract_domain("github.com")
        ));
        assert!(url_matches("www.example.com", "example.com"));
        assert!(!url_matches(
            &extract_domain("https://www.example.com"),
            &extract_domain("different.com")
        ));
    }
}
