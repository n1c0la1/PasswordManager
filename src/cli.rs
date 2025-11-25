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

    Get {},

    List {},

    Generate{},

    Remove{},

    ChangeMaster{},
    
}
