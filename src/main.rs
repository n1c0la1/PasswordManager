mod cli;
mod crypto;
mod errors;
mod session;
mod vault_entry_manager;
mod vault_file_manager;

use crate::errors::*;
use crate::session::Session;
use crate::session::*;
use crate::vault_file_manager::*;
use clap::Parser;
use cli::*;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    intro_animation();
    let current_session = Arc::new(Mutex::new(None::<Session>));

    // Background thread for AutoLock
    // just clones the Arc (which is a pointer), not the entire session!
    let session_clone = current_session.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let mut session_guard = session_clone.lock().unwrap();
            if let Some(session) = session_guard.as_mut() {
                if session.opened_vault.is_some() {
                    // Check for timeout (5 minutes)
                    if session.check_timeout(Duration::from_secs(20)) {
                        let name = session.vault_name.clone();
                        // Attempt to end session
                        match session.end_session() {
                            Ok(_) => {
                                handle_command_clear();
                                println!(
                                    "\n\nYou have been logged out. Last used vault was: '{}'.",
                                    name
                                );
                                io::stdout().flush().unwrap();
                            }
                            Err(e) => {
                                eprintln!("\nError in AutoLock: Could not end session: {}", e);
                            }
                        }
                    }
                } else {
                    /* Do nothing */
                }
            }
        }
    });

    'interactive_shell: loop {
        //println!("===================");
        println!("___________________");

        {
            let session_guard = current_session.lock().unwrap();
            println!(
                "Current vault: {}",
                match &*session_guard {
                    Some(session) => {
                        match &session.opened_vault {
                            Some(v) => v.get_name(),
                            None => "None",
                        }
                    }
                    None => "None",
                }
            );
        }
        println!("What action do you want to do? ");

        {
            let session_guard = current_session.lock().unwrap();
            if !check_vaults_exist() {
                eprintln!(
                    "\nHint: There are currently no vaults at all, consider using 'init' to create one!"
                );
            } else if !active_session(&*session_guard) {
                eprintln!(
                    "\nHint: There are currently no vaults open, consider using 'open <vault-name>'!"
                );
            }
        }

        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // split input like CLI-Args
        let args: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();

        // Clap expects args including program names as args[0]
        let mut args_with_prog = vec!["pw".to_string()];
        args_with_prog.extend(args);

        // parse input with parse
        let cli = match CLI::try_parse_from(args_with_prog) {
            Ok(cli) => cli,
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };

        {
            let mut session_guard = current_session.lock().unwrap();

            // Update activity before command
            if let Some(session) = session_guard.as_mut() {
                session.update_activity();
            }

            match cli.command {
                CommandCLI::Init { name } => {
                    match handle_command_init(name) {
                        Ok(()) => {
                            // nothing needed to do, vault gets created and closed immidiatly
                            /* Do nothing */
                        }
                        Err(VaultError::NameExists) => {
                            println!();
                            println!("Error: {}", VaultError::NameExists);
                            println!("Use a different name or open the existing vault.");
                            println!();
                        }
                        Err(e) => {
                            println!("Error: {e}");
                        }
                    }
                    continue 'interactive_shell;
                }

                CommandCLI::Add {
                    name,
                    username,
                    url,
                    notes,
                    password,
                } => {
                    if !active_session(&*session_guard) {
                        println!(
                            "There is no session active right now, consider using open <vault-name>!"
                        );
                        continue 'interactive_shell;
                    }
                    match handle_command_add(
                        &mut *session_guard,
                        name,
                        username,
                        url,
                        notes,
                        password,
                    ) {
                        Ok(()) => {
                            try_save(&mut *session_guard);
                        }
                        Err(SessionError::VaultError(VaultError::NoVaultOpen)) => {
                            println!("Error: {}", VaultError::NoVaultOpen);
                            println!("Consider using init or open <vault-name>!");
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                    continue 'interactive_shell;
                }

                CommandCLI::Get { name, show } => {
                    if !active_session(&*session_guard) {
                        println!(
                            "There is no session active right now, consider using open <vault-name>!"
                        );
                        continue 'interactive_shell;
                    }
                    match handle_command_get(&mut *session_guard, name, show) {
                        Ok(()) => { /* Do nothing */ }
                        Err(SessionError::VaultError(VaultError::NoVaultOpen)) => {
                            println!("No vault is active! Use init or open <vault-name>!");
                        }
                        Err(SessionError::VaultError(VaultError::CouldNotGetEntry)) => {
                            // already printing the name of non-existent vault in cli.rs
                            /* Do Nothing */
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                    continue 'interactive_shell;
                }

                CommandCLI::Getall { show } => {
                    if !active_session(&*session_guard) {
                        println!(
                            "There is no session active right now, consider using open <vault-name>!"
                        );
                        continue 'interactive_shell;
                    }
                    match handle_command_getall(&mut *session_guard, show) {
                        Ok(()) => { /* Do nothing */ }
                        Err(SessionError::VaultError(VaultError::NoVaultOpen)) => {
                            println!("No vault is active! Use init or open <vault-name>!");
                        }
                        Err(SessionError::VaultError(VaultError::CouldNotGetEntry)) => {
                            println!();
                            println!("The current vault does not have any entries yet!");
                            println!("Hint: Use add to create his first one!");
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                    continue 'interactive_shell;
                }

                CommandCLI::Delete { name } => {
                    if !active_session(&*session_guard) {
                        println!(
                            "There is no session active right now, consider using open <vault-name>!"
                        );
                        continue 'interactive_shell;
                    }
                    match handle_command_delete(&mut *session_guard, name) {
                        Ok(()) => {
                            /* Save, even though vault did not change, just to be sure. */
                            try_save(&mut *session_guard);
                        }
                        Err(SessionError::VaultError(VaultError::NoVaultOpen)) => {
                            println!("No vault is active! Use init or open <vault-name>!");
                        }
                        Err(SessionError::VaultError(VaultError::EntryNotFound)) => {
                            // because of printing name => in cli.rs
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                    continue 'interactive_shell;
                }

                CommandCLI::Deletevault {} => {
                    if !active_session(&*session_guard) {
                        println!("Due to RustPass's logic, you have to open your vault first!");
                        println!("Hint: Consider using open <vault-name>!");
                        continue 'interactive_shell;
                    }
                    match handle_command_deletevault(&mut *session_guard) {
                        Ok(()) => {
                            *session_guard = None;
                        }
                        Err(SessionError::VaultError(VaultError::AnyhowError(ref e)))
                            if e.to_string() == "Cancelled." =>
                        {
                            println!("\nDeletion cancelled.");
                        }
                        Err(SessionError::VaultError(VaultError::AnyhowError(ref e)))
                            if e.to_string() == "exit" =>
                        {
                            println!("Exiting...");
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                    continue 'interactive_shell;
                }

                CommandCLI::Generate { length, no_symbols } => {
                    match handle_command_generate(length, no_symbols) {
                        Ok(generated_pw) => {
                            println!("{}", generated_pw)
                        }
                        Err(e) => {
                            println!("Error: {}", e)
                        }
                    }
                    continue 'interactive_shell;
                }

                CommandCLI::ChangeMaster {} => {
                    if !active_session(&*session_guard) {
                        println!(
                            "There is no session active right now, consider using open <vault-name>!"
                        );
                        continue 'interactive_shell;
                    }
                    match handle_command_change_master(&mut *session_guard) {
                        Ok(()) => {
                            try_save(&mut *session_guard);
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                    continue 'interactive_shell;
                }

                CommandCLI::Edit { name } => {
                    if !active_session(&*session_guard) {
                        println!(
                            "There is no session active right now, consider using open <vault-name>!"
                        );
                        continue 'interactive_shell;
                    }
                    match handle_command_edit(&mut *session_guard, name) {
                        Ok(()) => {
                            /* Save, even though vault did not change, just to be sure. */
                            try_save(&mut *session_guard);
                        }
                        Err(e) => {
                            println!("Error: {}", e)
                        }
                    }
                    continue 'interactive_shell;
                }

                CommandCLI::Open { name } => {
                    match handle_command_open(name, &mut *session_guard) {
                        Ok(session) => {
                            if session.opened_vault.is_none() {
                                println!("Something went wrong!");
                                continue 'interactive_shell;
                            }
                            *session_guard = Some(session);
                        }
                        Err(SessionError::VaultError(VaultError::InvalidKey)) => {
                            println!("Error: Invalid password!")
                        }
                        Err(e) => {
                            println!("Error opening vault: {}", e);
                        }
                    }
                    continue 'interactive_shell;
                }
                CommandCLI::Close { force } => {
                    if !active_session(&*session_guard) {
                        println!(
                            "There is no session active right now, consider using open <vault-name>!"
                        );
                        continue 'interactive_shell;
                    }
                    match handle_command_close(&mut *session_guard, force) {
                        Ok(LoopCommand::Continue) => {
                            // if the user says yes to closing.
                            *session_guard = None;
                        }
                        Ok(LoopCommand::Cancel) => {
                            // if the user wishes to cancel the closing process.
                            /* Do nothing */
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                    continue 'interactive_shell;
                }

                CommandCLI::Vaults {} => {
                    handle_command_vaults(&*session_guard);
                }

                CommandCLI::Clear {} => {
                    handle_command_clear();
                }

                CommandCLI::Quit { force } => {
                    match handle_command_quit(force) {
                        Ok(LoopCommand::Continue) => {
                            if let Some(session) = &mut *session_guard {
                                match session.end_session() {
                                    Ok(()) => { /* Do nothing */ }
                                    Err(SessionError::SessionInactive) => { /* Ignore */ }
                                    Err(e) => {
                                        println!("Error: {}", e);
                                        continue 'interactive_shell;
                                    }
                                }
                            }
                            break 'interactive_shell;
                        }
                        Ok(LoopCommand::Cancel) => {
                            thread::sleep(Duration::from_millis(500));
                            continue 'interactive_shell;
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                            continue 'interactive_shell;
                        }
                    }
                }
            }

            // Update activity after command
            if let Some(session) = session_guard.as_mut() {
                session.update_activity();
            }
        } // End of session_guard scope
    }
}

fn try_save(current_session: &mut Option<Session>) {
    /* Save, even though vault did not change, just to be sure. */
    if let Some(session) = current_session {
        let spinner = spinner();
        spinner.set_message("Saving vault ...");
        spinner.enable_steady_tick(Duration::from_millis(80));
        match session.save() {
            Ok(()) => {
                spinner.finish_and_clear();
                println!("Vault saved.")
                /* Do nothing else */
            }
            Err(e) => {
                spinner.finish_and_clear();
                println!("Error: {}", e);
            }
        }
    }
}
