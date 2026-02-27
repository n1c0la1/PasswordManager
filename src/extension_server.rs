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
        (Some("fill"), Some(url)) => match session.lock() {
            Ok(session_guard) => match session_guard.as_ref() {
                Some(sess) => match_entries_by_url(sess, url),
                None => json!({"status": "error", "message": "No session open"}),
            },
            Err(_) => json!({"status": "error", "message": "Session state unavailable"}),
        },
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

    let vault = match session.opened_vault.as_ref() {
        Some(vault) => vault,
        None => return json!({"status": "error", "message": "No vault open"}),
    };
    let mut matches = Vec::new();

    for entry in &vault.entries {
        if let Some(entry_url) = entry.url() {
            if url_matches(entry_url, url) {
                matches.push(json!({
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault_entry_manager::{Entry, Vault};
    use serde_json::Value as JsonValue;
    use std::io::{Read, Write};
    use std::net::TcpStream;

    fn make_session_with_entries(entries: Vec<Entry>) -> Session {
        let mut session = Session::new("test_vault".to_string());
        let vault = Vault {
            name: "test_vault".to_string(),
            entries,
        };
        session.opened_vault = Some(vault);
        session
    }

    fn send_request(addr: &str, method: &str, body: Option<&str>) -> String {
        let mut stream = TcpStream::connect(addr).expect("connect failed");
        let body_bytes = body.unwrap_or("").as_bytes();
        let request = format!(
            "{method} / HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body_bytes.len(),
            body.unwrap_or("")
        );
        stream.write_all(request.as_bytes()).expect("write failed");

        let mut response = String::new();
        stream.read_to_string(&mut response).expect("read failed");
        response
    }

    fn parse_status(response: &str) -> u16 {
        response
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|code| code.parse::<u16>().ok())
            .unwrap_or(0)
    }

    fn parse_body_json(response: &str) -> JsonValue {
        let body = response.splitn(2, "\r\n\r\n").nth(1).unwrap_or("").trim();
        serde_json::from_str(body).unwrap_or_else(|_| json!({}))
    }

    fn with_server(
        session: Arc<Mutex<Option<Session>>>,
        token: String,
        method: &str,
        body: Option<&str>,
    ) -> String {
        let server = Server::http("127.0.0.1:0").expect("server start failed");
        let addr = server.server_addr().to_string();
        let handle = std::thread::spawn(move || {
            if let Some(request) = server.incoming_requests().next() {
                let _ = handle_request(request, session, token);
            }
        });

        let response = send_request(&addr, method, body);
        let _ = handle.join();
        response
    }

    #[test]
    fn test_rejects_non_post() {
        let session = Arc::new(Mutex::new(Some(Session::new("test_vault".to_string()))));
        let response = with_server(session, "token".to_string(), "GET", None);
        assert_eq!(parse_status(&response), 405);
    }

    #[test]
    fn test_invalid_token() {
        let session = Arc::new(Mutex::new(Some(Session::new("test_vault".to_string()))));
        let body = r#"{"action":"fill","url":"https://example.com","token":"bad"}"#;
        let response = with_server(session, "token".to_string(), "POST", Some(body));
        assert_eq!(parse_status(&response), 401);
        let json = parse_body_json(&response);
        assert_eq!(
            json.get("error").and_then(|v| v.as_str()),
            Some("invalid token")
        );
    }

    #[test]
    fn test_no_session_open() {
        let session = Arc::new(Mutex::new(None));
        let body = r#"{"action":"fill","url":"https://example.com","token":"token"}"#;
        let response = with_server(session, "token".to_string(), "POST", Some(body));
        assert_eq!(parse_status(&response), 200);
        let json = parse_body_json(&response);
        assert_eq!(json.get("status").and_then(|v| v.as_str()), Some("error"));
        assert_eq!(
            json.get("message").and_then(|v| v.as_str()),
            Some("No session open")
        );
    }

    #[test]
    fn test_single_match() {
        let entries = vec![Entry::new(
            "entry1".to_string(),
            Some("user1".to_string()),
            Some("pass1".to_string()),
            Some("https://example.com".to_string()),
            None,
        )];
        let session = make_session_with_entries(entries);
        let session = Arc::new(Mutex::new(Some(session)));
        let body = r#"{"action":"fill","url":"https://example.com/login","token":"token"}"#;
        let response = with_server(session, "token".to_string(), "POST", Some(body));
        let json = parse_body_json(&response);
        assert_eq!(json.get("status").and_then(|v| v.as_str()), Some("ok"));
        assert_eq!(json.get("mode").and_then(|v| v.as_str()), Some("single"));
        assert_eq!(json.get("username").and_then(|v| v.as_str()), Some("user1"));
        assert_eq!(json.get("password").and_then(|v| v.as_str()), Some("pass1"));
    }

    #[test]
    fn test_multiple_match() {
        let entries = vec![
            Entry::new(
                "entry1".to_string(),
                Some("user1".to_string()),
                Some("pass1".to_string()),
                Some("https://example.com".to_string()),
                None,
            ),
            Entry::new(
                "entry2".to_string(),
                Some("user2".to_string()),
                Some("pass2".to_string()),
                Some("https://example.com/login".to_string()),
                None,
            ),
        ];
        let session = make_session_with_entries(entries);
        let session = Arc::new(Mutex::new(Some(session)));
        let body = r#"{"action":"fill","url":"https://example.com","token":"token"}"#;
        let response = with_server(session, "token".to_string(), "POST", Some(body));
        let json = parse_body_json(&response);
        assert_eq!(json.get("status").and_then(|v| v.as_str()), Some("ok"));
        assert_eq!(json.get("mode").and_then(|v| v.as_str()), Some("multiple"));
        let entries = json.get("entries").and_then(|v| v.as_array());
        assert!(entries.is_some(), "Expected entries array in response");
        assert_eq!(entries.map(|values| values.len()), Some(2));
    }
}
