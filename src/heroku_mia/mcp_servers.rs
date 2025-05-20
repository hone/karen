use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct McpServerResponse {
    pub id: String,
    pub app_id: String,
    pub process_type: String,
    pub process_command: String,
    pub created_at: String,
    pub updated_at: String,
    pub tools: Vec<ToolDetails>,
    pub server_status: ServerStatus,
    pub primitives_status: PrimitivesStatus,
    pub namespace: String,
}

#[derive(Deserialize, Debug)]
pub struct ToolDetails {
    pub name: String,
    pub namespaced_name: String,
    pub description: String,
    pub input_schema: Value,
    pub annotations: Annotations,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Annotations {
    title: String,
    read_only_hint: bool,
    destructive_hint: bool,
    idempotent_hint: bool,
    open_world_hint: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ServerStatus {
    Registered,
    Disconnected,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PrimitivesStatus {
    Syncing,
    Synced,
    Error,
}
