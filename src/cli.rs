use crate::vault_manager;

use clap::{Parser, Subcommand, command};
use indicatif::{self, ProgressBar, ProgressStyle};
use std::{path::PathBuf, time::Duration};
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

