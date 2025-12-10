use crate::vault_manager::*;

use clap::{Parser, Subcommand, command};
use indicatif::{self, ProgressBar, ProgressStyle};
use rpassword;
use serde::{Deserialize, Serialize};
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
