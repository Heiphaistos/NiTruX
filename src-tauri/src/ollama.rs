//! Local AI assistant through Ollama's HTTP API on 127.0.0.1:11434. The
//! address is fixed: nothing the frontend sends can point this at another
//! host. Plain HTTP/1.1 over std's TcpStream -- two endpoints do not
//! justify an HTTP client dependency.

use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

const PORT: u16 = 11434;

fn http(port: u16, method: &str, path: &str, body: &str, timeout: Duration) -> Result<String, String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port))
        .map_err(|_| "Ollama ne répond pas sur 127.0.0.1:11434 : lancez-le (ollama serve, ou systemctl start ollama)".to_string())?;
    stream.set_read_timeout(Some(timeout)).ok();
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(request.as_bytes()).map_err(|e| format!("envoi à Ollama impossible : {e}"))?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).map_err(|e| format!("réponse d'Ollama illisible : {e}"))?;
    let raw = String::from_utf8_lossy(&raw);
    let (head, payload) = raw.split_once("\r\n\r\n").ok_or("réponse d'Ollama incomplète")?;
    let status = head.lines().next().unwrap_or("");
    let payload = if head.to_ascii_lowercase().contains("transfer-encoding: chunked") { dechunk(payload) } else { payload.to_string() };
    if !status.contains(" 200 ") {
        return Err(format!("Ollama a répondu « {status} » : {}", payload.trim()));
    }
    Ok(payload)
}

fn dechunk(mut body: &str) -> String {
    let mut out = String::new();
    while let Some((size, rest)) = body.split_once("\r\n") {
        let Ok(n) = usize::from_str_radix(size.trim(), 16) else { break };
        if n == 0 || rest.len() < n {
            break;
        }
        out.push_str(&rest[..n]);
        body = rest[n..].trim_start_matches("\r\n");
    }
    out
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct OllamaStatus {
    pub installed: bool,
    pub running: bool,
    pub models: Vec<String>,
    pub error: Option<String>,
}

fn status_on(port: u16) -> OllamaStatus {
    let installed = crate::subprocess::binary_in_path("ollama");
    match http(port, "GET", "/api/tags", "", Duration::from_secs(5)) {
        Ok(json) => {
            let models = serde_json::from_str::<serde_json::Value>(&json)
                .ok()
                .and_then(|v| v.get("models")?.as_array().cloned())
                .unwrap_or_default()
                .iter()
                .filter_map(|m| m.get("name")?.as_str().map(str::to_string))
                .collect();
            OllamaStatus { installed: true, running: true, models, error: None }
        }
        Err(e) => OllamaStatus { installed, running: false, models: Vec::new(), error: Some(e) },
    }
}

#[tauri::command]
pub fn ollama_status() -> OllamaStatus {
    status_on(PORT)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

fn validate_model(model: &str) -> Result<(), String> {
    if model.is_empty() || model.len() > 128 || !model.chars().all(|c| c.is_ascii_alphanumeric() || "._:/-".contains(c)) {
        return Err(format!("nom de modèle invalide : {model}"));
    }
    Ok(())
}

const SYSTEM_PROMPT: &str = "Tu es l'assistant de NiTruX, un outil de maintenance et de diagnostic Linux. \
Réponds en français, simplement. Quand tu proposes une commande, explique ce qu'elle fait et préviens \
avant toute commande qui supprime des données ou demande les droits root.";

fn chat_on(port: u16, model: &str, messages: &[ChatMessage]) -> Result<String, String> {
    validate_model(model)?;
    let mut all = vec![ChatMessage { role: "system".into(), content: SYSTEM_PROMPT.into() }];
    all.extend(messages.iter().filter(|m| m.role == "user" || m.role == "assistant").cloned());
    let body = serde_json::json!({ "model": model, "messages": all, "stream": false }).to_string();
    let json = http(port, "POST", "/api/chat", &body, Duration::from_secs(600))?;
    serde_json::from_str::<serde_json::Value>(&json)
        .ok()
        .and_then(|v| v.get("message")?.get("content")?.as_str().map(str::to_string))
        .ok_or_else(|| "réponse d'Ollama sans message".to_string())
}

#[tauri::command]
pub async fn ollama_chat(model: String, messages: Vec<ChatMessage>) -> Result<String, String> {
    // A generation takes seconds to minutes: off the main thread.
    tauri::async_runtime::spawn_blocking(move || chat_on(PORT, &model, &messages))
        .await
        .map_err(|e| format!("tâche interrompue : {e}"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    /// One-shot fake Ollama: answers the first request with `response`.
    fn fake_server(response: &'static str) -> (u16, std::thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let handle = std::thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let mut buf = [0u8; 8192];
            let n = sock.read(&mut buf).unwrap();
            sock.write_all(response.as_bytes()).unwrap();
            String::from_utf8_lossy(&buf[..n]).into_owned()
        });
        (port, handle)
    }

    #[test]
    fn lists_models_from_api_tags() {
        let (port, server) = fake_server("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"models\":[{\"name\":\"llama3.2:3b\"},{\"name\":\"qwen2.5:7b\"}]}");
        let status = status_on(port);
        assert!(server.join().unwrap().starts_with("GET /api/tags HTTP/1.1"));
        assert!(status.running);
        assert_eq!(status.models, vec!["llama3.2:3b", "qwen2.5:7b"]);
    }

    #[test]
    fn chat_sends_the_system_prompt_and_reads_a_chunked_answer() {
        let (port, server) = fake_server("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n1f\r\n{\"message\":{\"content\":\"Bonjour\"\r\n2\r\n}}\r\n0\r\n\r\n");
        let reply = chat_on(port, "llama3.2:3b", &[ChatMessage { role: "user".into(), content: "Salut".into() }]).unwrap();
        let request = server.join().unwrap();
        assert!(request.contains("\"role\":\"system\""));
        assert!(request.contains("Salut"));
        assert_eq!(reply, "Bonjour");
    }

    #[test]
    fn reports_a_stopped_ollama_and_rejects_odd_model_names() {
        let free = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let status = status_on(free);
        assert!(!status.running);
        assert!(status.error.unwrap().contains("ollama serve"));
        assert!(validate_model("llama3; rm -rf /").is_err());
        assert!(validate_model("hf.co/user/model:Q4_K_M").is_ok());
    }
}
