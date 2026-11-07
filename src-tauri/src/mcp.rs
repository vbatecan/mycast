//! User-configured local MCP clients. Servers start only on explicit Connect.
use rmcp::{
    RoleClient, ServiceExt,
    model::{CallToolRequestParams, Tool},
    service::RunningService,
    transport::TokioChildProcess,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::{AppHandle, Manager, State, ipc::Channel};
use tokio::process::Command;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServerConfig {
    pub id: String,
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

struct Connection {
    client: RunningService<RoleClient, ()>,
    tools: Vec<Tool>,
}
#[derive(Default)]
pub struct McpState {
    configs: Mutex<Vec<ServerConfig>>,
    connections: Mutex<BTreeMap<String, Arc<Connection>>>,
    approvals: Mutex<HashMap<String, tokio::sync::oneshot::Sender<bool>>>,
    operation: tokio::sync::Mutex<()>,
    load_error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerView {
    config: ServerConfig,
    connected: bool,
    tools: Vec<String>,
}

#[derive(Clone)]
pub struct ToolSpec {
    pub alias: String,
    pub server_id: String,
    pub server_name: String,
    pub name: String,
    pub description: String,
    pub schema: Value,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolApproval {
    pub id: String,
    pub server: String,
    pub tool: String,
    pub arguments: Value,
}

fn path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|path| path.join("mcp.json"))
        .map_err(|_| "Could not locate MCP settings.".into())
}
fn validate(config: &ServerConfig) -> Result<(), String> {
    if config.name.trim().is_empty()
        || config.name.len() > 100
        || config.command.trim().is_empty()
        || config.command.len() > 4096
        || config.command.contains(['\0', '\r', '\n'])
        || config.args.len() > 64
        || config
            .args
            .iter()
            .any(|arg| arg.len() > 4096 || arg.contains('\0'))
    {
        return Err("Enter a server name, executable, and at most 64 arguments.".into());
    }
    Ok(())
}
impl McpState {
    pub fn stop(&self) {
        self.approvals.lock().unwrap().clear();
        for (_, connection) in std::mem::take(&mut *self.connections.lock().unwrap()) {
            connection.client.cancellation_token().cancel();
        }
    }
    pub fn for_app(app: &AppHandle) -> Self {
        Self::load(app).unwrap_or_else(|error| Self {
            load_error: Some(error),
            ..Self::default()
        })
    }
    pub fn load(app: &AppHandle) -> Result<Self, String> {
        let configs = match std::fs::read(path(app)?) {
            Ok(bytes) if bytes.len() <= 512_000 => {
                serde_json::from_slice::<Vec<ServerConfig>>(&bytes)
                    .map_err(|_| "MCP settings are unreadable.".to_string())?
            }
            Ok(_) => return Err("MCP settings are too large.".into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => vec![],
            Err(_) => return Err("Could not read MCP settings.".into()),
        };
        for config in &configs {
            validate(config)?;
        }
        if configs.len() > 12 {
            return Err("At most 12 MCP servers are supported.".into());
        }
        Ok(Self {
            configs: Mutex::new(configs),
            ..Self::default()
        })
    }
}

fn views(state: &McpState) -> Vec<ServerView> {
    let connections = state.connections.lock().unwrap();
    state
        .configs
        .lock()
        .unwrap()
        .iter()
        .map(|config| {
            let connection = connections
                .get(&config.id)
                .filter(|connection| !connection.client.is_closed());
            ServerView {
                config: config.clone(),
                connected: connection.is_some(),
                tools: connection
                    .map(|connection| {
                        connection
                            .tools
                            .iter()
                            .map(|tool| tool.name.to_string())
                            .collect()
                    })
                    .unwrap_or_default(),
            }
        })
        .collect()
}
fn persist(app: &AppHandle, configs: &[ServerConfig]) -> Result<(), String> {
    let target = path(app)?;
    std::fs::create_dir_all(
        target
            .parent()
            .ok_or("MCP configuration directory is unavailable.")?,
    )
    .map_err(|_| "Could not create MCP settings directory.")?;
    let bytes =
        serde_json::to_vec_pretty(configs).map_err(|_| "Could not serialize MCP settings.")?;
    let temporary = target.with_extension("json.tmp");
    std::fs::write(&temporary, bytes)
        .and_then(|_| std::fs::rename(temporary, target))
        .map_err(|_| "Could not save MCP settings.".into())
}

#[tauri::command]
pub fn get_mcp_servers(state: State<'_, McpState>) -> Result<Vec<ServerView>, String> {
    if let Some(error) = &state.load_error {
        return Err(error.clone());
    }
    Ok(views(&state))
}

#[tauri::command]
pub async fn save_mcp_server(
    app: AppHandle,
    state: State<'_, McpState>,
    mut config: ServerConfig,
) -> Result<Vec<ServerView>, String> {
    crate::modules::require("ai")?;
    if let Some(error) = &state.load_error {
        return Err(error.clone());
    }
    validate(&config)?;
    let _operation = state.operation.lock().await;
    if state.connections.lock().unwrap().contains_key(&config.id) {
        return Err("Disconnect this server before editing it.".into());
    }
    let mut configs = state.configs.lock().unwrap().clone();
    if config.id.is_empty() {
        if configs.len() >= 12 {
            return Err("At most 12 MCP servers are supported.".into());
        }
        config.id = uuid::Uuid::new_v4().to_string();
        configs.push(config);
    } else if let Some(saved) = configs.iter_mut().find(|saved| saved.id == config.id) {
        *saved = config;
    } else {
        return Err("MCP server not found.".into());
    }
    persist(&app, &configs)?;
    *state.configs.lock().unwrap() = configs;
    Ok(views(&state))
}

#[tauri::command]
pub async fn remove_mcp_server(
    app: AppHandle,
    state: State<'_, McpState>,
    id: String,
) -> Result<Vec<ServerView>, String> {
    if let Some(error) = &state.load_error {
        return Err(error.clone());
    }
    let _operation = state.operation.lock().await;
    let mut configs = state.configs.lock().unwrap().clone();
    configs.retain(|config| config.id != id);
    persist(&app, &configs)?;
    *state.configs.lock().unwrap() = configs;
    if let Some(connection) = state.connections.lock().unwrap().remove(&id) {
        connection.client.cancellation_token().cancel();
    }
    Ok(views(&state))
}

async fn connect(config: &ServerConfig) -> Result<Connection, String> {
    validate(config)?;
    let mut command = Command::new(config.command.trim());
    command.args(&config.args).kill_on_drop(true);
    #[cfg(windows)]
    command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    let transport = TokioChildProcess::new(command).map_err(|_| {
        "Could not start MCP server. Check the executable and arguments.".to_string()
    })?;
    let client = tokio::time::timeout(Duration::from_secs(30), ().serve(transport))
        .await
        .map_err(|_| "MCP connection timed out.")?
        .map_err(|_| "MCP initialization failed. Check the server command and protocol.")?;
    let tools = tokio::time::timeout(Duration::from_secs(15), client.list_all_tools())
        .await
        .map_err(|_| "MCP tool discovery timed out.")?
        .map_err(|_| "Could not list MCP tools.")?;
    if tools.len() > 64 {
        return Err("This server exposes more than 64 tools.".into());
    }
    Ok(Connection { client, tools })
}

#[tauri::command]
pub async fn connect_mcp_server(
    state: State<'_, McpState>,
    id: String,
) -> Result<Vec<ServerView>, String> {
    crate::modules::require("ai")?;
    let _operation = state.operation.lock().await;
    let config = state
        .configs
        .lock()
        .unwrap()
        .iter()
        .find(|config| config.id == id)
        .cloned()
        .ok_or("MCP server not found.")?;
    if state
        .connections
        .lock()
        .unwrap()
        .get(&id)
        .is_some_and(|connection| !connection.client.is_closed())
    {
        return Ok(views(&state));
    }
    let connection = connect(&config).await?;
    {
        let mut connections = state.connections.lock().unwrap();
        if let Err(error) = crate::modules::require("ai") {
            connection.client.cancellation_token().cancel();
            return Err(error);
        }
        connections.insert(id, Arc::new(connection));
    }
    Ok(views(&state))
}

#[tauri::command]
pub async fn disconnect_mcp_server(
    state: State<'_, McpState>,
    id: String,
) -> Result<Vec<ServerView>, String> {
    let _operation = state.operation.lock().await;
    if let Some(connection) = state.connections.lock().unwrap().remove(&id) {
        connection.client.cancellation_token().cancel();
    }
    Ok(views(&state))
}

pub fn tools(app: &AppHandle) -> Vec<ToolSpec> {
    let state = app.state::<McpState>();
    let connections = state.connections.lock().unwrap();
    let configs = state.configs.lock().unwrap();
    let mut tools = vec![];
    for (server_index, (id, connection)) in connections.iter().enumerate() {
        if connection.client.is_closed() {
            continue;
        }
        let Some(config) = configs.iter().find(|config| &config.id == id) else {
            continue;
        };
        for (tool_index, tool) in connection.tools.iter().enumerate() {
            if tools.len() == 64 {
                return tools;
            }
            tools.push(ToolSpec {
                alias: format!("mcp_{server_index}_{tool_index}"),
                server_id: id.clone(),
                server_name: config.name.clone(),
                name: tool.name.to_string(),
                description: tool
                    .description
                    .as_deref()
                    .unwrap_or(&tool.name)
                    .chars()
                    .take(4000)
                    .collect(),
                schema: Value::Object((*tool.input_schema).clone()),
            });
        }
    }
    tools
}

struct ApprovalGuard {
    app: AppHandle,
    id: String,
}
impl Drop for ApprovalGuard {
    fn drop(&mut self) {
        self.app
            .state::<McpState>()
            .approvals
            .lock()
            .unwrap()
            .remove(&self.id);
    }
}

pub async fn call(
    app: &AppHandle,
    spec: &ToolSpec,
    arguments: Value,
    output: &Channel<crate::ai::AiEvent>,
) -> Result<Value, String> {
    crate::modules::require("ai")?;
    let arguments = arguments
        .as_object()
        .cloned()
        .ok_or("Tool arguments must be a JSON object.")?;
    if serde_json::to_vec(&arguments)
        .map_err(|_| "Invalid tool arguments.")?
        .len()
        > 32_000
    {
        return Err("Tool arguments are too large.".into());
    }
    let id = uuid::Uuid::new_v4().to_string();
    let (send, receive) = tokio::sync::oneshot::channel();
    app.state::<McpState>()
        .approvals
        .lock()
        .unwrap()
        .insert(id.clone(), send);
    let _guard = ApprovalGuard {
        app: app.clone(),
        id: id.clone(),
    };
    output
        .send(crate::ai::AiEvent::Status(
            "Waiting for tool approval…".into(),
        ))
        .map_err(|_| "AI view closed.")?;
    output
        .send(crate::ai::AiEvent::ToolApproval(ToolApproval {
            id,
            server: spec.server_name.clone(),
            tool: spec.name.clone(),
            arguments: Value::Object(arguments.clone()),
        }))
        .map_err(|_| "AI view closed.")?;
    let approved = tokio::time::timeout(Duration::from_secs(120), receive)
        .await
        .map_err(|_| "Tool approval timed out.")?
        .map_err(|_| "Tool approval was cancelled.")?;
    if !approved {
        return Ok(json!({ "error": "The user declined this tool call." }));
    }
    crate::modules::require("ai")?;
    output
        .send(crate::ai::AiEvent::Status(format!(
            "Using {} · {}…",
            spec.server_name, spec.name
        )))
        .map_err(|_| "AI view closed.")?;
    let connection = app
        .state::<McpState>()
        .connections
        .lock()
        .unwrap()
        .get(&spec.server_id)
        .cloned()
        .ok_or("MCP server is disconnected.")?;
    let result = tokio::time::timeout(
        Duration::from_secs(60),
        connection
            .client
            .call_tool(CallToolRequestParams::new(spec.name.clone()).with_arguments(arguments)),
    )
    .await
    .map_err(|_| "MCP tool call timed out.")?
    .map_err(|_| "MCP tool call failed.")?;
    let value = serde_json::to_value(result).map_err(|_| "Invalid MCP tool result.")?;
    if serde_json::to_vec(&value)
        .map_err(|_| "Invalid MCP result.")?
        .len()
        > 128_000
    {
        return Err("MCP result is too large. Request a smaller result.".into());
    }
    Ok(value)
}

#[tauri::command]
pub fn mcp_approve_tool(state: State<'_, McpState>, id: String, allow: bool) -> Result<(), String> {
    let send = state
        .approvals
        .lock()
        .unwrap()
        .remove(&id)
        .ok_or("This tool request is no longer pending.")?;
    send.send(allow)
        .map_err(|_| "This tool request was cancelled.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn local_stdio_initializes_discovers_and_calls_a_fixture_tool() {
        let config = ServerConfig {
            id: "fixture".into(),
            name: "Fixture".into(),
            command: "node".into(),
            args: vec![
                "--input-type=module".into(),
                "-e".into(),
                include_str!("../../tests/fixtures/mcp-server.mjs").into(),
            ],
        };
        let connection = connect(&config).await.unwrap();
        assert_eq!(connection.tools.len(), 1);
        assert_eq!(connection.tools[0].name, "echo");
        let result = connection
            .client
            .call_tool(CallToolRequestParams::new("echo").with_arguments(
                serde_json::from_value(json!({"message":"fixture response"})).unwrap(),
            ))
            .await
            .unwrap();
        let value = serde_json::to_value(result).unwrap();
        assert_eq!(value["content"][0]["text"], "fixture response");
        connection.client.cancel().await.unwrap();
    }
    #[test]
    fn invalid_server_configuration_is_rejected() {
        let config = ServerConfig {
            id: "".into(),
            name: "".into(),
            command: "node".into(),
            args: vec![],
        };
        assert!(validate(&config).is_err());
        let config = ServerConfig {
            name: "fixture".into(),
            command: "node\nmalformed".into(),
            ..config
        };
        assert!(validate(&config).is_err());
    }
}
