#![cfg(feature = "oauth")]

// OAuth 2.0 + PKCE flow for Claude authentication.
// This module is only compiled when the `oauth` feature is enabled.

use crate::error::{CswitchError, Result};
use crate::keychain;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

const AUTH_URL: &str = "https://console.anthropic.com/oauth/authorize";
const TOKEN_URL: &str = "https://console.anthropic.com/oauth/token";
const CLIENT_ID: &str = "claude-code";
const REDIRECT_PORT: u16 = 19832;

fn generate_pkce() -> (String, String) {
    let mut rng = rand::thread_rng();
    let verifier_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    let verifier = URL_SAFE_NO_PAD.encode(&verifier_bytes);

    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(hasher.finalize());

    (verifier, challenge)
}

pub fn run_oauth_flow(profile_name: &str) -> Result<()> {
    let (verifier, challenge) = generate_pkce();
    let redirect_uri = format!("http://localhost:{REDIRECT_PORT}/callback");

    let auth_url = format!(
        "{AUTH_URL}?response_type=code&client_id={CLIENT_ID}&redirect_uri={}&code_challenge={challenge}&code_challenge_method=S256&scope=user:inference",
        urlencoding_encode(&redirect_uri)
    );

    println!("Opening browser for authentication...");
    if open::open(&auth_url).is_err() {
        println!("Please open this URL in your browser:\n{auth_url}");
    }

    // Listen for the callback
    let listener = TcpListener::bind(format!("127.0.0.1:{REDIRECT_PORT}"))
        .map_err(|e| CswitchError::OAuth(format!("Failed to bind listener: {e}")))?;

    let (stream, _) = listener
        .accept()
        .map_err(|e| CswitchError::OAuth(format!("Failed to accept connection: {e}")))?;

    let mut reader = BufReader::new(&stream);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .map_err(|e| CswitchError::OAuth(format!("Failed to read request: {e}")))?;

    // Extract code from GET /callback?code=...
    let code = request_line
        .split_whitespace()
        .nth(1)
        .and_then(|path| {
            path.split('?')
                .nth(1)?
                .split('&')
                .find(|p| p.starts_with("code="))
                .map(|p| p.trim_start_matches("code=").to_string())
        })
        .ok_or_else(|| CswitchError::OAuth("No authorization code received".into()))?;

    // Send success response to browser
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h2>Authentication successful!</h2><p>You can close this tab.</p></body></html>";
    let mut writer = stream;
    let _ = writer.write_all(response.as_bytes());

    // Exchange code for token
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| CswitchError::OAuth(format!("Failed to create runtime: {e}")))?;

    let token_json = rt.block_on(async {
        let client = reqwest::Client::new();
        let resp = client
            .post(TOKEN_URL)
            .form(&[
                ("grant_type", "authorization_code"),
                ("client_id", CLIENT_ID),
                ("code", &code),
                ("redirect_uri", &redirect_uri),
                ("code_verifier", &verifier),
            ])
            .send()
            .await
            .map_err(|e| CswitchError::OAuth(format!("Token request failed: {e}")))?;

        let body = resp
            .text()
            .await
            .map_err(|e| CswitchError::OAuth(format!("Failed to read token response: {e}")))?;

        Ok::<String, CswitchError>(body)
    })?;

    keychain::set_oauth_token(profile_name, &token_json)?;

    Ok(())
}

fn urlencoding_encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                String::from(b as char)
            }
            _ => format!("%{:02X}", b),
        })
        .collect()
}
