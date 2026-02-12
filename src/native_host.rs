#![allow(dead_code)]
// dead code allowed because this is only used in native host mode

use serde_json::json;
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use url::Url;

use crate::cli::handle_command_get_by_url;
use crate::session::Session;

const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB max message size to prevent abuse
// The message is the JSON object received from the extension

fn read_message() -> Option<serde_json::Value> {
    let mut len_buf = [0u8; 4];
    io::stdin().read_exact(&mut len_buf).ok()?;
    let len = u32::from_ne_bytes(len_buf) as usize;

    // Validate message size to prevent OOM attacks
    if len > MAX_MESSAGE_SIZE {
        eprintln!(
            "Warning: Message size {} exceeds maximum allowed {}",
            len, MAX_MESSAGE_SIZE
        );
        // eprint can be read from the extension console for debugging in the browser
        return None;
    }

    let mut buf = vec![0u8; len];
    io::stdin().read_exact(&mut buf).ok()?;
    serde_json::from_slice(&buf).ok()
}

fn send_message(value: &serde_json::Value) -> io::Result<()> {
    let s = serde_json::to_string(value)?;
    let len = (s.len() as u32).to_ne_bytes();
    let mut stdout = io::stdout();
    stdout.write_all(&len)?;
    stdout.write_all(s.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

fn normalize_origin(input: &str) -> Option<String> {
    let url = Url::parse(input).ok()?;
    match url.scheme() {
        "http" | "https" => {}
        _ => return None,
    }
    let host = url.host_str()?;
    Some(format!("{}://{}", url.scheme(), host))
}

pub fn run(session_arc: Arc<Mutex<Option<Session>>>) {
    eprintln!("Native host thread started");

    while let Some(msg) = read_message() {
        eprintln!("Received message: {}", msg);

        let origin = msg
            .get("origin")
            .and_then(|v| v.as_str())
            .and_then(normalize_origin);

        let response = match origin {
            Some(o) => {
                eprintln!("Looking up entries for: {}", o);

                // Lock and read the session
                // Handle mutex poison gracefully to prevent thread panic
                // mutex poisoning can occur if another thread panics while holding the lock (other thread might be autolock)
                let session_guard = match session_arc.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => {
                        eprintln!("Warning: Session mutex was poisoned, attempting recovery");
                        poisoned.into_inner()
                    }
                };

                match &*session_guard {
                    Some(session) => {
                        eprintln!("Session is active, searching for entries");
                        match handle_command_get_by_url(session, &o) {
                            Ok(entries) => {
                                eprintln!("Found {} entries", entries.len());

                                if entries.is_empty() {
                                    json!({ "found": false })
                                } else if entries.len() == 1 {
                                    // single entry found: return directly
                                    let entry = &entries[0];
                                    json!({
                                        "found": true,
                                        "entryname": entry.get_entry_name(),
                                        "username": entry.get_user_name(),
                                        "password": entry.get_password()
                                    })
                                } else {
                                    // Multiple entries: return all for selection
                                    let entry_list: Vec<_> = entries
                                        .iter()
                                        .map(|e| {
                                            json!({
                                                "entryname": e.get_entry_name(),
                                                "username": e.get_user_name(),
                                                "password": e.get_password()
                                            })
                                        })
                                        .collect();
                                    json!({ "entries": entry_list })
                                }
                            }
                            Err(e) => {
                                eprintln!("Error getting entries: {:?}", e);
                                json!({ "error": format!("Failed to get entries: {:?}", e) })
                            }
                        }
                    }
                    None => {
                        eprintln!("No active session");
                        json!({ "error": "no session active" })
                    }
                }
            }
            None => {
                eprintln!("Invalid origin");
                json!({ "error": "invalid origin" })
            }
        };

        let _ = send_message(&response);
    }
}
