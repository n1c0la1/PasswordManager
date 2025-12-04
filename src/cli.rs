use clap::{Parser, Subcommand, command};
use std::path::PathBuf;
use rpassword;
use serde::{Deserialize, Serialize};


#[derive(Parser)]
#[command(name = "pw")]
pub struct CLI {
    #[arg(short, long)]
    pub database: PathBuf,

    #[command(subcommand)]
    pub command: CommandCLI,
}

#[derive(Subcommand)]
pub enum CommandCLI {
    /// Initializes a new PasswordManager.
    Init{},

    /// Adds a new password to database.
    Add {
        name: String,

        #[arg(short, long)]
        username: Option<String>,

        #[arg(short, long)]
        url: Option<String>,

        #[arg(short, long)]
        password: Option<String>,

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

    /// Modify a given password
    //
    Modify{},
    
}

/*fn main() {
    let cli = CLI::parse();

    match cli.command {
        Command::Init {} => todo!(),
        Command::Add { name, username, url, notes , password} => todo!(),
        Command::Get {  } => todo!(),
        Command::List {  } => todo!(),
        Command::Remove {  } => todo!(),
        Command::Generate {  } => todo!(),
        Command::ChangeMaster {  } => todo!(),
        Command::Modify {  } => todo!(),
    }
}*/
