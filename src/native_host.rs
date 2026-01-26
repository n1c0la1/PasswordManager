use std::io::{self, Read, Write};
use serde_json::json;
use url::Url;

// Reads a message from stdin and parse it as JSON
fn read_message() -> Option<serde_json::Value> {
    let mut len_buf = [0u8; 4];
    io::stdin().read_exact(&mut len_buf).ok()?;
    let len = u32::from_ne_bytes(len_buf) as usize;

    let mut buf = vec![0u8; len];
    io::stdin().read_exact(&mut buf).ok()?;
    serde_json::from_slice(&buf).ok()
}

// Sends a JSON message to stdout
fn send_message(value: &serde_json::Value) -> io::Result<()> {
    let s = serde_json::to_string(value)?;
    let len = (s.len() as u32).to_ne_bytes();
    let mut stdout = io::stdout();
    stdout.write_all(&len)?;
    stdout.write_all(s.as_bytes())?;
    stdout.flush()?;
    Ok(())
}

// Normalizes the origin URL to a standard format (e.g. from "https://example.com/path" to "https://example.com")
fn normalize_origin(input: &str) -> Option<String> {
    let url = Url::parse(input).ok()?;

    // Only support http(s) pages
    match url.scheme() {
        "http" | "https" => {}
        _ => return None,
    }

    let host = url.host_str()?;
    Some(format!("{}://{}", url.scheme(), host))
}

// Test credentials (for demonstration purposes)
fn get_test_credentials(origin: &str) -> Option<(String, String)> {
    match origin {
        "https://github.com" => Some(("octocat".to_string(), "github_token_123".to_string())),
        "https://example.com" => Some(("testuser".to_string(), "testpass123".to_string())),
        _ => None,
    }
}

// for testing purposes I added simple cmd handling like we do in the main script but it was simplified for my testing project, needs actual implementation and communication with the cli script
fn main() {
    eprintln!("Native host started");

    // Main message loop
    while let Some(msg) = read_message() {
        eprintln!("Received message: {}", msg);

        let origin = msg
            .get("origin")
            .and_then(|v| v.as_str())
            .and_then(normalize_origin);

        // simplified response handling for testing with console output
        let response = match origin {
            Some(o) => {
                if let Some((username, password)) = get_test_credentials(&o) {
                    eprintln!("Found test credentials for {}", o);
                    json!({
                        "found": true,
                        "username": username,
                        "password": password
                    })
                } else {
                    eprintln!("No credentials found for {}", o);
                    json!({ "found": false })
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
