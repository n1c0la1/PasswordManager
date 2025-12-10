use crate::vault_manager::*;

use clap::{Parser, Subcommand, command};
use indicatif::{self, ProgressBar, ProgressStyle};
use std::{path::PathBuf, time::Duration};
use std::io::{self, Write};
use rpassword;
use serde::{Deserialize, Serialize};


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
        #[arg(short='n', long)]
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
        #[arg(short='f', long)]
        force: bool,
    },

}

pub fn handle_command_init(mut current_vault: Option<Vault>, name: Option<String>, spinner: ProgressBar) {
    println!("\nInitializing new vault: ");
    
    let vault_name = if let Some(n) = name {
                    n
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
        println!("\n---------------\nDefine the Master-Password for {}:", vault_name);
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
    
    match initialize_vault(vault_name.clone(), key) {
        Ok(vault) => {
            current_vault = Some(vault);
    
            spinner.enable_steady_tick(Duration::from_millis(80));
            spinner.set_message(" Creating vault...");
    
            //unwrapping here is unproblematic, because the User just initialized a vault
            //if this vault was successfully made, it is the current_vault -> not None
            current_vault.as_ref().unwrap().save();
    
            spinner.finish_and_clear();
    
            println!("\nVault '{}' created successfully! \n", vault_name);
        }
        Err(e) => {println!("Error: {}", e);}
    }
}