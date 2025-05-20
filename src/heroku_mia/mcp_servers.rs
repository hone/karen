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

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ServerStatus {
    Registered,
    Disconnected,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum PrimitivesStatus {
    Syncing,
    Synced,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mcp_server_response_deserialization() {
        let json_data = json!([
          {
            "id": "15ac9382-5804-49ac-bf94-8bab2f4b62ce",
            "app_id": "434eb878-6bc1-4677-928d-80d27047ad5a",
            "process_type": "mcp",
            "process_command": "python -m src.stdio_server",
            "created_at": "2025-05-07T16:44:34.259Z",
            "updated_at": "2025-05-07T16:44:38.291Z",
            "namespace": "acute-partridge",
            "server_status": "registered",
            "primitives_status": "synced",
            "tools": [
                {
                    "name": "my_tool",
                    "namespaced_name": "acute-partridge.my_tool",
                    "description": "A sample tool",
                    "input_schema": {},
                    "annotations": {
                        "title": "My Tool",
                        "readOnlyHint": false,
                        "destructiveHint": false,
                        "idempotentHint": true,
                        "openWorldHint": false
                    }
                }
            ]
          }
        ]);

        let responses: Vec<McpServerResponse> = serde_json::from_value(json_data).unwrap();
        let response = &responses[0];

        assert_eq!(response.id, "15ac9382-5804-49ac-bf94-8bab2f4b62ce");
        assert_eq!(response.app_id, "434eb878-6bc1-4677-928d-80d27047ad5a");
        assert_eq!(response.process_type, "mcp");
        assert_eq!(response.process_command, "python -m src.stdio_server");
        assert_eq!(response.created_at, "2025-05-07T16:44:34.259Z");
        assert_eq!(response.updated_at, "2025-05-07T16:44:38.291Z");
        assert_eq!(response.namespace, "acute-partridge");
        assert_eq!(response.server_status, ServerStatus::Registered);
        assert_eq!(response.primitives_status, PrimitivesStatus::Synced);
        assert_eq!(response.tools.len(), 1);

        let tool = &response.tools[0];
        assert_eq!(tool.name, "my_tool");
        assert_eq!(tool.namespaced_name, "acute-partridge.my_tool");
        assert_eq!(tool.description, "A sample tool");
        assert_eq!(tool.input_schema, json!({}));
        assert_eq!(tool.annotations.title, "My Tool");
        assert_eq!(tool.annotations.read_only_hint, false);
        assert_eq!(tool.annotations.destructive_hint, false);
        assert_eq!(tool.annotations.idempotent_hint, true);
        assert_eq!(tool.annotations.open_world_hint, false);
    }
}
