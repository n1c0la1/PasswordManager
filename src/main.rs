mod cli;
mod vault_manager;

use clap::{Parser, Command, Subcommand, command};
use serde_json::Value; //imports value type (represents json data)
use std::fs; //imports rusts file system module
use cli::*;
use vault_manager::*;
use std::path::PathBuf;
use std::io::{self, Write};


fn main() {
    let string_from_json =
        fs::read_to_string("src/passwords_file.json").expect("could not read file");
    let json_data: Value = serde_json::from_str(&string_from_json).expect("invalid json");

    println!("{json_data}");

    let mut current_vault: Option<Vault> = None;

    password_manager::intro_animation();
    println!("Hello, world!");

    'interactive_shell: loop {
        println!("What action do you want to do? ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // Die Eingabe splitten wie CLI-Args
        let args: Vec<String> = input
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        // Clap erwartet args inklusive Programmnamen als args[0]
        let mut args_with_prog = vec!["pw".to_string()];
        args_with_prog.extend(args);

        // Mit Clap parsen
        let cli = match CLI::try_parse_from(args_with_prog) {
            Ok(cli) => cli,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };
        match cli.command {
            CommandCLI::Init { name } => {
                println!("Initializing new vault: \n");

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
                        println!("Vault '{}' created successfully! \n", vault_name);
                        current_vault = Some(vault);
                    }
                    Err(e) => {println!("Error: {}", e);}
                }

                //unwrapping here is unproblematic, because the User just initialized a vault
                //if this vault was successfully made, it is the current_vault -> not None
                current_vault.as_ref().unwrap().safe();
            },

            CommandCLI::Add { name, username, url, notes , password} => todo!(),
            CommandCLI::Get { name, show } => todo!(),
            CommandCLI::List { vault, show  } => todo!(),
            CommandCLI::Remove { name } => todo!(),
            CommandCLI::Generate { length, no_symbols } => todo!(),
            CommandCLI::ChangeMaster {  } => todo!(),
            CommandCLI::Modify { name } => todo!(),
            CommandCLI::Quit { force } => { 
                if force {
                    println!("Quitting RustPass...");
                    break;
                } 

                print!("Are you sure you want to quit? (y/n): ");
                io::stdout().flush().unwrap();


                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();

                if input.trim().eq_ignore_ascii_case("y") {
                    println!("Quitting RustPass...");
                    current_vault.unwrap().close();
                    break 'interactive_shell;
                } else {
                    println!("Cancelled.");
                    io::stdout().flush().unwrap();
                }
            },
        }
    }
}
