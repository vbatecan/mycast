use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Mutex, time::Duration};
use tauri::{AppHandle, Manager, State, ipc::Channel};

#[derive(Default)]
pub struct AiState(pub Mutex<HashMap<String, tokio::sync::oneshot::Sender<()>>>);
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
    pub provider: String,
    pub model: String,
    pub base_url: String,
    #[serde(default)]
    pub has_key: bool,
}
const PROVIDERS: [&str; 6] = ["openai", "claude", "gemini", "custom", "ollama", "lmstudio"];
fn base(provider: &str) -> &str {
    match provider {
        "claude" => "https://api.anthropic.com/v1",
        "gemini" => "https://generativelanguage.googleapis.com/v1beta",
        "ollama" => "http://localhost:11434/v1",
        "lmstudio" => "http://localhost:1234/v1",
        _ => "https://api.openai.com/v1",
    }
}
fn validate(config: &AiConfig) -> Result<(), String> {
    if !PROVIDERS.contains(&config.provider.as_str()) {
        return Err("Unknown AI provider.".into());
    }
    if config.model.trim().is_empty() || config.model.len() > 200 {
        return Err("Enter a model ID (up to 200 characters).".into());
    }
    let url = tauri::Url::parse(&config.base_url).map_err(|_| "Enter a valid API base URL.")?;
    let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if !(url.scheme() == "https" || (url.scheme() == "http" && local))
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(
            "Use HTTPS, or HTTP on localhost, without credentials, query, or fragment.".into(),
        );
    }
    if matches!(config.provider.as_str(), "openai" | "claude" | "gemini")
        && config.base_url.trim_end_matches('/') != base(&config.provider)
    {
        return Err("Use Custom OpenAI-compatible for a custom endpoint.".into());
    }
    Ok(())
}
#[cfg(windows)]
fn credential(provider: &str, endpoint: &str) -> Result<keyring::Entry, String> {
    use sha2::{Digest, Sha256};
    let account = format!(
        "{provider}-{:x}",
        Sha256::digest(endpoint.trim_end_matches('/').as_bytes())
    );
    keyring::Entry::new("Mycast AI", &account)
        .map_err(|_| "Could not access Windows Credential Manager.".into())
}
fn read_key(config: &AiConfig) -> Result<String, String> {
    #[cfg(windows)]
    {
        match credential(&config.provider, &config.base_url)?.get_password() {
            Ok(key) => Ok(key),
            Err(keyring::Error::NoEntry) => Ok(String::new()),
            Err(_) => Err("Could not read the API key from Windows Credential Manager.".into()),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = config;
        Ok(String::new())
    }
}
#[tauri::command]
pub fn get_ai_config(app: AppHandle, provider: String) -> Result<AiConfig, String> {
    if !PROVIDERS.contains(&provider.as_str()) {
        return Err("Unknown AI provider.".into());
    }
    let path = app
        .path()
        .app_config_dir()
        .map_err(|_| "Configuration directory unavailable")?
        .join(format!("ai-{provider}.json"));
    let mut config = match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map_err(|_| "AI configuration is unreadable. Save it again.")?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => AiConfig {
            base_url: base(&provider).into(),
            provider,
            model: String::new(),
            has_key: false,
        },
        Err(_) => return Err("Could not read AI configuration.".into()),
    };
    config.has_key = !read_key(&config)?.is_empty();
    Ok(config)
}
#[tauri::command]
pub fn save_ai_config(
    app: AppHandle,
    config: AiConfig,
    api_key: Option<String>,
    remove_key: bool,
) -> Result<AiConfig, String> {
    validate(&config)?;
    if let Some(key) = &api_key
        && (key.len() > 4096 || key.contains(['\r', '\n']))
    {
        return Err("Invalid API key.".into());
    }
    #[cfg(windows)]
    {
        let entry = credential(&config.provider, &config.base_url)?;
        if remove_key {
            match entry.delete_credential() {
                Ok(()) | Err(keyring::Error::NoEntry) => (),
                Err(_) => return Err("Could not delete saved key.".into()),
            }
        } else if let Some(key) = api_key.filter(|key| !key.is_empty()) {
            entry
                .set_password(&key)
                .map_err(|_| "Could not save key in Windows Credential Manager.")?;
        }
    }
    #[cfg(not(windows))]
    if api_key.is_some_and(|key| !key.is_empty()) || remove_key {
        return Err("API key storage currently requires Windows.".into());
    }
    let directory = app
        .path()
        .app_config_dir()
        .map_err(|_| "Configuration directory unavailable")?;
    std::fs::create_dir_all(&directory).map_err(|_| "Could not create configuration directory")?;
    let target = directory.join(format!("ai-{}.json", config.provider));
    let temp = target.with_extension("tmp");
    std::fs::write(
        &temp,
        serde_json::to_vec(&config).map_err(|_| "Could not serialize AI settings")?,
    )
    .map_err(|_| "Could not save AI settings")?;
    std::fs::rename(temp, target).map_err(|_| "Could not save AI settings")?;
    get_ai_config(app, config.provider)
}

#[derive(Clone, Serialize)]
#[serde(tag = "type", content = "text", rename_all = "kebab-case")]
pub enum AiEvent {
    Text(String),
    Thinking(String),
    Status(String),
    ToolApproval(crate::mcp::ToolApproval),
}

// Only provider-supplied public stream fields are displayed; never fabricate reasoning.
fn deltas(value: &Value) -> Result<Vec<AiEvent>, String> {
    if value.get("error").is_some() || value["type"] == "error" {
        return Err(
            "The provider reported a streaming error. Check your model, quota, and API settings."
                .into(),
        );
    }
    let mut result = vec![];
    let delta = &value["choices"][0]["delta"];
    for (field, thinking) in [
        ("content", false),
        ("reasoning_content", true),
        ("reasoning", true),
    ] {
        if let Some(text) = delta[field].as_str() {
            result.push(if thinking {
                AiEvent::Thinking(text.into())
            } else {
                AiEvent::Text(text.into())
            });
        }
    }
    if let Some(text) = value["delta"]["text"].as_str() {
        result.push(AiEvent::Text(text.into()));
    }
    if let Some(text) = value["delta"]["thinking"].as_str() {
        result.push(AiEvent::Thinking(text.into()));
    }
    if let Some(parts) = value["candidates"][0]["content"]["parts"].as_array() {
        for part in parts {
            if let Some(text) = part["text"].as_str() {
                result.push(if part["thought"] == true {
                    AiEvent::Thinking(text.into())
                } else {
                    AiEvent::Text(text.into())
                });
            }
        }
    }
    if value["promptFeedback"]["blockReason"].is_string() {
        return Err("The provider blocked this prompt.".into());
    }
    Ok(result)
}
#[cfg(test)]
fn frame(bytes: &[u8], output: &Channel<AiEvent>) -> Result<bool, String> {
    frame_collect(bytes, output, None)
}
fn frame_collect(
    bytes: &[u8],
    output: &Channel<AiEvent>,
    mut round: Option<&mut crate::ai_tools::Round>,
) -> Result<bool, String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "Invalid UTF-8 in AI stream")?;
    let data = text
        .lines()
        .filter_map(|line| line.strip_prefix("data:").map(str::trim_start))
        .collect::<Vec<_>>()
        .join("\n");
    if data == "[DONE]" {
        return Ok(true);
    }
    if data.is_empty() {
        return Ok(false);
    }
    let value: Value = serde_json::from_str(&data)
        .map_err(|_| "Invalid response from AI endpoint. Expected an SSE JSON stream.")?;
    if let Some(round) = round.as_deref_mut() {
        round.collect(&value);
    }
    for event in deltas(&value)? {
        if let AiEvent::Text(text) = &event {
            if let Some(round) = round.as_deref_mut() {
                round.text.push_str(text);
            }
        }
        output.send(event).map_err(|_| "AI view closed")?;
    }
    Ok(value["type"] == "message_stop"
        || value["choices"][0]["finish_reason"].is_string()
        || value["candidates"][0]["finishReason"].is_string())
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConversationMessage {
    role: String,
    content: String,
}

fn conversation_messages(
    prompt: String,
    history: Vec<ConversationMessage>,
) -> Result<Vec<ConversationMessage>, String> {
    if prompt.trim().is_empty() || prompt.len() > 32_000 {
        return Err("Enter a prompt up to 32,000 bytes.".into());
    }
    if history.len() > 32
        || history.len() % 2 != 0
        || history
            .iter()
            .map(|message| message.content.len())
            .sum::<usize>()
            > 256_000
    {
        return Err("Conversation context is too large. Start a new conversation.".into());
    }
    for (index, message) in history.iter().enumerate() {
        if message.role != if index % 2 == 0 { "user" } else { "assistant" }
            || message.content.trim().is_empty()
        {
            return Err("Invalid conversation history.".into());
        }
    }
    let mut messages = history;
    messages.push(ConversationMessage {
        role: "user".into(),
        content: prompt,
    });
    Ok(messages)
}

fn conversation_body(config: &AiConfig, messages: &[ConversationMessage]) -> Value {
    if config.provider == "gemini" {
        return json!({ "contents": messages.iter().map(|message| json!({
            "role": if message.role == "assistant" { "model" } else { "user" },
            "parts": [{"text": message.content}],
        })).collect::<Vec<_>>() });
    }
    let mut body = json!({ "model": config.model, "messages": messages, "stream": true });
    if config.provider == "claude" {
        body["max_tokens"] = json!(4096);
    }
    body
}

async fn stream(
    config: AiConfig,
    messages: Vec<ConversationMessage>,
    output: Channel<AiEvent>,
    app: Option<AppHandle>,
) -> Result<(), String> {
    validate(&config)?;
    let key = read_key(&config)?;
    if matches!(config.provider.as_str(), "openai" | "claude" | "gemini") && key.is_empty() {
        return Err("Add your API key in Preferences → AI.".into());
    }
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(15))
        .build()
        .map_err(|_| "Could not initialize AI connection")?;
    let root = config.base_url.trim_end_matches('/');
    let mut url =
        tauri::Url::parse(&format!("{root}/chat/completions")).map_err(|_| "Invalid endpoint")?;
    let mut body = conversation_body(&config, &messages);
    let tools = app.as_ref().map(crate::mcp::tools).unwrap_or_default();
    crate::ai_tools::definitions(&mut body, &config.provider, &tools);
    if config.provider == "claude" {
        url = tauri::Url::parse(&format!("{root}/messages")).map_err(|_| "Invalid endpoint")?;
    }
    if config.provider == "gemini" {
        url = tauri::Url::parse(&format!("{root}/models/")).map_err(|_| "Invalid endpoint")?;
        url.path_segments_mut()
            .map_err(|_| "Invalid endpoint")?
            .pop_if_empty()
            .push(&format!("{}:streamGenerateContent", config.model));
        url.set_query(Some("alt=sse"));
    }
    for round_index in 0..8 {
        let mut round = crate::ai_tools::Round::default();
        let mut request = client.post(url.clone()).json(&body);
        if !key.is_empty() {
            request = match config.provider.as_str() {
                "claude" => request
                    .header("x-api-key", &key)
                    .header("anthropic-version", "2023-06-01"),
                "gemini" => request.header("x-goog-api-key", &key),
                _ => request.bearer_auth(&key),
            };
        }
        output
            .send(AiEvent::Status("Connecting…".into()))
            .map_err(|_| "AI view closed")?;
        let mut response = request
            .send()
            .await
            .map_err(|_| "Could not connect to AI provider. Check the endpoint and network.")?;
        if !response.status().is_success() {
            return Err(format!(
                "AI provider returned HTTP {}. Check your key, model, quota, and endpoint.",
                response.status().as_u16()
            ));
        }
        output
            .send(AiEvent::Status("Waiting for the model…".into()))
            .map_err(|_| "AI view closed")?;
        let mut buffer = vec![];
        let mut total = 0;
        'response: while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| "AI stream was interrupted. You can retry.")?
        {
            total += chunk.len();
            if total > 2_000_000 {
                return Err("AI response exceeded the 2 MB limit.".into());
            }
            buffer.extend_from_slice(&chunk);
            // Handle both LF and CRLF SSE separators, even across network chunks.
            loop {
                let boundary = buffer
                    .windows(2)
                    .position(|v| v == b"\n\n")
                    .map(|p| (p, 2))
                    .into_iter()
                    .chain({
                        buffer
                            .windows(4)
                            .position(|v| v == b"\r\n\r\n")
                            .map(|p| (p, 4))
                    })
                    .min_by_key(|(position, _)| *position);
                let Some((end, size)) = boundary else {
                    break;
                };
                if frame_collect(&buffer[..end], &output, Some(&mut round))? {
                    buffer.clear();
                    break 'response;
                }
                buffer.drain(..end + size);
            }
        }
        if !buffer.is_empty() {
            frame_collect(&buffer, &output, Some(&mut round))?;
        }

        if round.calls.is_empty() {
            return Ok(());
        }
        let app = app
            .as_ref()
            .ok_or("MCP tools are not available in this request.")?;
        let mut results = vec![];
        for (index, call) in &round.calls {
            let mut call = call.clone();
            if call.id.is_empty() && config.provider != "gemini" {
                call.id = format!("mcp_call_{round_index}_{index}");
            }
            let result = if let Some(tool) = tools.iter().find(|tool| tool.alias == call.name) {
                match serde_json::from_str::<Value>(if call.arguments.is_empty() {
                    "{}"
                } else {
                    &call.arguments
                }) {
                    Ok(arguments) => crate::mcp::call(app, tool, arguments, &output)
                        .await
                        .unwrap_or_else(|error| json!({"error":error})),
                    Err(_) => json!({"error":"Invalid tool arguments. Send a JSON object."}),
                }
            } else {
                json!({"error":"This tool is not connected or available."})
            };
            results.push((call, result));
        }
        crate::ai_tools::append_results(&mut body, &config.provider, &round, &results)?;
        if !round.text.is_empty() {
            output
                .send(AiEvent::Text("\n\n".into()))
                .map_err(|_| "AI view closed")?;
        }
    }
    Err("The AI reached the tool-call limit. Ask a more focused question.".into())
}
#[tauri::command]
pub fn cancel_ai(state: State<'_, AiState>, id: String) {
    if let Some(cancel) = state.0.lock().unwrap().remove(&id) {
        let _ = cancel.send(());
    }
}
#[tauri::command]
pub async fn ask_ai(
    app: AppHandle,
    state: State<'_, AiState>,
    id: String,
    provider: String,
    prompt: String,
    history: Option<Vec<ConversationMessage>>,
    output: Channel<AiEvent>,
) -> Result<(), String> {
    crate::modules::require("ai")?;
    let messages = conversation_messages(prompt, history.unwrap_or_default())?;
    let config = get_ai_config(app.clone(), provider)?;
    let (tx, rx) = tokio::sync::oneshot::channel();
    {
        let mut active = state.0.lock().unwrap();
        if !active.is_empty() {
            return Err("Stop the current AI request first.".into());
        }
        active.insert(id.clone(), tx);
    }
    let result = tokio::select! {
        _ = rx => Ok(()),
        result = tokio::time::timeout(Duration::from_secs(180), stream(config, messages, output, Some(app))) => result.unwrap_or_else(|_| Err("AI request timed out. Try again or choose a faster model.".into())),
    };
    state.0.lock().unwrap().remove(&id);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conversation_history_is_mapped_for_each_provider() {
        let messages = conversation_messages(
            "What name did I give you?".into(),
            vec![
                ConversationMessage {
                    role: "user".into(),
                    content: "My name is Nytri".into(),
                },
                ConversationMessage {
                    role: "assistant".into(),
                    content: "Hello Nytri".into(),
                },
            ],
        )
        .unwrap();
        for provider in ["openai", "custom", "ollama", "lmstudio", "claude", "gemini"] {
            let config = AiConfig {
                provider: provider.into(),
                model: "fixture".into(),
                base_url: base(provider).into(),
                has_key: false,
            };
            let body = conversation_body(&config, &messages);
            if provider == "gemini" {
                assert_eq!(body["contents"].as_array().unwrap().len(), 3);
                assert_eq!(body["contents"][1]["role"], "model");
                assert_eq!(body["contents"][0]["parts"][0]["text"], "My name is Nytri");
            } else {
                assert_eq!(body["messages"].as_array().unwrap().len(), 3);
                assert_eq!(body["messages"][1]["role"], "assistant");
                assert_eq!(body["messages"][0]["content"], "My name is Nytri");
            }
        }
        assert!(
            conversation_messages(
                "hi".into(),
                vec![ConversationMessage {
                    role: "system".into(),
                    content: "bad".into()
                }]
            )
            .is_err()
        );
        assert!(
            conversation_messages(
                "hi".into(),
                vec![
                    ConversationMessage {
                        role: "assistant".into(),
                        content: "bad".into()
                    },
                    ConversationMessage {
                        role: "user".into(),
                        content: "bad".into()
                    }
                ]
            )
            .is_err()
        );
    }
    #[test]
    fn provider_deltas() {
        assert!(
            matches!(&deltas(&json!({"choices":[{"delta":{"content":"Hi"}}]})).unwrap()[0], AiEvent::Text(s) if s == "Hi")
        );
        assert!(matches!(
            &deltas(&json!({"delta":{"thinking":"preview"}})).unwrap()[0],
            AiEvent::Thinking(_)
        ));
        assert!(matches!(
            &deltas(&json!({"candidates":[{"content":{"parts":[{"text":"Hi"}]}}]})).unwrap()[0],
            AiEvent::Text(_)
        ));
        assert!(deltas(&json!({"error":{"message":"secret"}})).is_err());
    }
    #[test]
    fn sse_frames_and_errors() {
        use std::sync::Arc;
        let values = Arc::new(Mutex::new(Vec::<Value>::new()));
        let collected = values.clone();
        let output = Channel::new(move |body| {
            collected
                .lock()
                .unwrap()
                .push(body.deserialize::<Value>().unwrap());
            Ok(())
        });
        assert!(!frame(b": ping", &output).unwrap());
        assert!(
            !frame(
                b"event: content_block_delta\r\ndata: {\"delta\":{\"text\":\"Hello\"}}",
                &output
            )
            .unwrap()
        );
        assert!(frame(b"data: [DONE]", &output).unwrap());
        assert!(frame(b"data: {bad}", &output).is_err());
        assert!(
            frame(b"data: {\"error\":{\"message\":\"secret\"}}", &output)
                .unwrap_err()
                .find("secret")
                .is_none()
        );
        assert_eq!(values.lock().unwrap()[0]["text"], "Hello");
    }
    #[tokio::test]
    async fn local_endpoint_streams_fragmented_unicode() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            sync::Arc,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(10)))
                .unwrap();
            let mut request = vec![];
            let mut buffer = [0u8; 4096];
            loop {
                let size = socket.read(&mut buffer).unwrap();
                if size == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..size]);
                if let Some(end) = request.windows(4).position(|v| v == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&request[..end]);
                    let length: usize = headers
                        .lines()
                        .find_map(|line| {
                            line.to_lowercase()
                                .strip_prefix("content-length:")
                                .map(|value| value.trim().parse().unwrap())
                        })
                        .unwrap_or(0);
                    if request.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            let text = String::from_utf8(request).unwrap();
            assert!(text.starts_with("POST /v1/chat/completions "));
            assert!(text.contains("\"stream\":true"));
            let sent: Value = serde_json::from_str(text.split_once("\r\n\r\n").unwrap().1).unwrap();
            assert_eq!(sent["messages"].as_array().unwrap().len(), 3);
            assert_eq!(sent["messages"][0]["content"], "Remember orchid");
            assert_eq!(sent["messages"][1]["content"], "Orchid noted");
            let body =
                "data: {\"choices\":[{\"delta\":{\"content\":\"你好\"}}]}\r\n\r\ndata: [DONE]\n\n";
            write!(socket,"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).unwrap();
            for chunk in body.as_bytes().chunks(1) {
                socket.write_all(chunk).unwrap();
            }
        });
        let values = Arc::new(Mutex::new(Vec::<Value>::new()));
        let collected = values.clone();
        let output = Channel::new(move |body| {
            collected
                .lock()
                .unwrap()
                .push(body.deserialize::<Value>().unwrap());
            Ok(())
        });
        let config = AiConfig {
            provider: "custom".into(),
            model: "fixture".into(),
            base_url: format!("http://{address}/v1"),
            has_key: false,
        };
        stream(
            config,
            conversation_messages(
                "Which plant?".into(),
                vec![
                    ConversationMessage {
                        role: "user".into(),
                        content: "Remember orchid".into(),
                    },
                    ConversationMessage {
                        role: "assistant".into(),
                        content: "Orchid noted".into(),
                    },
                ],
            )
            .unwrap(),
            output,
            None,
        )
        .await
        .unwrap();
        server.join().unwrap();
        assert!(
            values
                .lock()
                .unwrap()
                .iter()
                .any(|value| value["type"] == "text" && value["text"] == "你好")
        );
    }
    #[test]
    fn endpoint_validation() {
        let mut c = AiConfig {
            provider: "custom".into(),
            model: "test".into(),
            base_url: "http://localhost:1234/v1".into(),
            has_key: false,
        };
        assert!(validate(&c).is_ok());
        for bad in [
            "http://example.com/v1",
            "https://user:secret@example.com",
            "file:///tmp",
            "https://example.com?key=secret",
        ] {
            c.base_url = bad.into();
            assert!(validate(&c).is_err());
        }
    }
}

#[tauri::command]
pub fn open_ai_link(url: String) -> Result<(), String> {
    crate::modules::require("ai")?;
    crate::search::validate_url(&url)?;
    crate::search::open_uri(&url)
}
