use crate::session::Session;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tiny_http::{Request, Response, Server};

// Extension server to handle requests from the web extension
pub fn run(session: Arc<Mutex<Option<Session>>>, token: String) {
    let listener = match Server::http("127.0.0.1:9123") {
        Ok(server) => {
            eprintln!("Extension server listening on http://127.0.0.1:9123");
            server
        }
        Err(e) => {
            eprintln!("Failed to start extension server: {}", e);
            return;
        }
    };

    for request in listener.incoming_requests() {
        // Clone the token and session Arc for the thread, so we can move them in
        let token_clone = token.clone();
        let session_clone = session.clone();

        std::thread::spawn(move || {
            if let Err(e) = handle_request(request, session_clone, token_clone) {
                eprintln!("Error handling request: {}", e);
            }
        });
    }
}

fn handle_request(
    request: Request,
    session: Arc<Mutex<Option<Session>>>,
    token: String,
) -> Result<(), Box<dyn std::error::Error>> {
    if request.method() != &tiny_http::Method::Post {
        // Only POST requests are allowed (POST = for sending data)
        request.respond(Response::from_string("Method not allowed").with_status_code(405))?;
        return Ok(());
    }

    // Read request body
    let mut content = Vec::new();
    let mut request = request;
    request.as_reader().read_to_end(&mut content)?;

    let body: Value = serde_json::from_slice(&content)?;

    // Validate token
    let provided_token = body.get("token").and_then(|v| v.as_str());
    if provided_token != Some(&token) {
        request.respond(
            Response::from_string(json!({"error": "invalid token"}).to_string())
                .with_status_code(401),
        )?;
        return Ok(());
    }

    // Get action and URL
    let action = body.get("action").and_then(|v| v.as_str());
    let url = body.get("url").and_then(|v| v.as_str());

    let response = match (action, url) {
        (Some("fill"), Some(url)) => {
            let session_guard = session.lock().unwrap();
            match session_guard.as_ref() {
                Some(sess) => match_entries_by_url(sess, url),
                None => json!({"status": "error", "message": "No session open"}),
            }
        }
        _ => json!({"error": "Invalid request"}),
    };

    let header = match tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..])
    {
        Ok(h) => h,
        Err(_) => return Err("Invalid Content-Type header".into()),
    };

    request.respond(Response::from_string(response.to_string()).with_header(header))?;
    Ok(())
}

fn match_entries_by_url(session: &Session, url: &str) -> Value {
    use crate::cli::url_matches;

    if session.opened_vault.is_none() {
        return json!({"status": "error", "message": "No vault open"});
    }

    let vault = session.opened_vault.as_ref().unwrap();
    let mut matches = Vec::new();

    for entry in &vault.entries {
        if let Some(entry_url) = entry.url() {
            if url_matches(entry_url, url) {
                matches.push(json!({
                    "id": entry.id,
                    "username": entry.username(),
                    "password": entry.password(),
                    "url": entry.url(),
                }));
            }
        }
    }

    match matches.len() {
        0 => json!({"status": "not_found"}),
        1 => {
            let entry = &matches[0];
            json!({
                "status": "ok",
                "mode": "single",
                "username": entry.get("username"),
                "password": entry.get("password"),
            })
        }
        _ => json!({"status": "ok", "mode": "multiple", "entries": matches}),
    }
}
