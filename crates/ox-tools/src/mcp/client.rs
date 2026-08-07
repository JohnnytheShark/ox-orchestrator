use crate::error::ToolError;
use crate::mcp::protocol::{
    JsonRpcRequest, JsonRpcResponse, McpToolCallResult, McpToolsListResult,
};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::{oneshot, Mutex};

pub struct McpClient {
    server_name: String,
    request_tx: tokio::sync::mpsc::Sender<String>,
    pending_requests: Arc<Mutex<HashMap<i64, oneshot::Sender<JsonRpcResponse>>>>,
    next_id: AtomicI64,
}

impl McpClient {
    /// Launches an MCP server subprocess communicating over stdio.
    pub async fn launch_stdio(
        server_name: impl Into<String>,
        command: &str,
        args: &[String],
        env: HashMap<String, String>,
    ) -> Result<Self, ToolError> {
        let name = server_name.into();
        let mut child = Command::new(command)
            .args(args)
            .envs(env)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| ToolError::ExecutionFailed {
                tool: name.clone(),
                message: format!("Failed to spawn MCP process '{}': {}", command, e),
            })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| ToolError::ExecutionFailed {
                tool: name.clone(),
                message: "Failed to open stdin for MCP server".to_string(),
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ToolError::ExecutionFailed {
                tool: name.clone(),
                message: "Failed to open stdout for MCP server".to_string(),
            })?;

        let (req_tx, mut req_rx) = tokio::sync::mpsc::channel::<String>(32);
        let pending: Arc<Mutex<HashMap<i64, oneshot::Sender<JsonRpcResponse>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let pending_clone = pending.clone();

        // Stdin writer task
        tokio::spawn(async move {
            let mut writer = stdin;
            while let Some(msg) = req_rx.recv().await {
                if writer.write_all(msg.as_bytes()).await.is_err() {
                    break;
                }
                if writer.write_all(b"\n").await.is_err() {
                    break;
                }
                if writer.flush().await.is_err() {
                    break;
                }
            }
        });

        // Stdout reader task
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(&line) {
                    if let Some(Value::Number(num)) = &resp.id {
                        if let Some(id) = num.as_i64() {
                            let mut map = pending_clone.lock().await;
                            if let Some(tx) = map.remove(&id) {
                                let _ = tx.send(resp);
                            }
                        }
                    }
                }
            }
        });

        let client = Self {
            server_name: name,
            request_tx: req_tx,
            pending_requests: pending,
            next_id: AtomicI64::new(1),
        };

        // Initialize MCP handshake
        client.initialize().await?;

        Ok(client)
    }

    /// Sends a JSON-RPC request and awaits the matched response.
    pub async fn call_rpc(&self, method: &str, params: Option<Value>) -> Result<Value, ToolError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let req = JsonRpcRequest::new(id, method, params);
        let json_str = serde_json::to_string(&req)?;

        let (tx, rx) = oneshot::channel();
        {
            let mut map = self.pending_requests.lock().await;
            map.insert(id, tx);
        }

        self.request_tx
            .send(json_str)
            .await
            .map_err(|_| ToolError::ExecutionFailed {
                tool: self.server_name.clone(),
                message: "MCP channel closed".to_string(),
            })?;

        let response = rx
            .await
            .map_err(|_| ToolError::McpTimeout(self.server_name.clone()))?;

        if let Some(err) = response.error {
            return Err(ToolError::McpProtocolError {
                code: err.code,
                message: err.message,
            });
        }

        Ok(response.result.unwrap_or(Value::Null))
    }

    /// Sends standard MCP initialize sequence.
    async fn initialize(&self) -> Result<(), ToolError> {
        let init_params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "ox-orchestrator",
                "version": "0.1.0"
            }
        });

        self.call_rpc("initialize", Some(init_params)).await?;

        // Send notifications/initialized (no response expected)
        let note = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::Null,
            method: "notifications/initialized".to_string(),
            params: None,
        };
        let _ = self.request_tx.send(serde_json::to_string(&note)?).await;

        Ok(())
    }

    /// Discovers all available tools on this MCP server.
    pub async fn list_tools(&self) -> Result<McpToolsListResult, ToolError> {
        let val = self.call_rpc("tools/list", None).await?;
        let res: McpToolsListResult = serde_json::from_value(val)?;
        Ok(res)
    }

    /// Invokes a specific tool on this MCP server.
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: &Value,
    ) -> Result<McpToolCallResult, ToolError> {
        let params = serde_json::json!({
            "name": name,
            "arguments": arguments,
        });

        let val = self.call_rpc("tools/call", Some(params)).await?;
        let res: McpToolCallResult = serde_json::from_value(val)?;
        Ok(res)
    }

    pub fn server_name(&self) -> &str {
        &self.server_name
    }
}
