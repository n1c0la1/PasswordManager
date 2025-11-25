use clap::{Parser, Subcommand};
use std::path::PathBuf;
use rpassword;


#[derive(Parser)]
#[command(name = "pw")]
struct CLI {
    #[arg(short, long)]
    database: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initializes a new PasswordManager
    Init{},

    /// Adds a new password to database.
    Add {
        name: String,

        #[arg(short, long)]
        username: Option<String>,

        #[arg(short, long)]
        url: Option<String>,

        #[arg(short, long)]
        notes: Option<String>,
    },

    /// Get an Entry of the Database.
    // maybe implement parameter: with or without the password visible
    Get {},

    /// List all Entrys.
    // maybe implement filters e.g. all passwords with that email, or on that URL.
    List {},

    /// Generate a password.
    // maybe implement interaction (abfrage) if with special cases, numbers etc.
    Generate{}, 

    /// Remove an entry from Database.
    Remove{},

    /// Change the Masterpassword.
    // implement not visible, old password required. Verschlüsselt Vault sofort
    ChangeMaster{},
    
}

struct PasswordEntry {
    name: String,
    username: Option<String>,
    password: Option<String>,
    url: Option<String>,
    notes: Option<String>,
} 

