use serde::{Deserialize, Deserializer};
use serde_json::Value;

#[derive(Deserialize, PartialEq, Debug)]
pub struct McpServerResponse {
    pub id: String,
    pub app_id: String,
    pub process_type: String,
    pub process_command: String,
    pub created_at: String,
    pub updated_at: String,
    pub namespace: String,
    pub server_status: ServerStatus,
    pub primitives_status: PrimitivesStatus,
    pub tools: Vec<ToolDetails>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct ToolDetails {
    pub name: String,
    pub namespaced_name: String,
    pub description: Option<String>,
    pub input_schema: Value,
    #[serde(deserialize_with = "deserialize_annotations")]
    pub annotations: Option<Annotations>,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Annotations {
    title: Option<String>,
    read_only_hint: bool,
    destructive_hint: bool,
    idempotent_hint: bool,
    open_world_hint: bool,
}

fn deserialize_annotations<'de, D>(deserializer: D) -> Result<Option<Annotations>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<Value> = Option::deserialize(deserializer)?;

    match value {
        Some(v) => {
            if v.is_object() && v.as_object().map_or(false, |obj| obj.is_empty()) {
                Ok(None)
            } else {
                Annotations::deserialize(v)
                    .map(|a| Some(a))
                    .map_err(serde::de::Error::custom)
            }
        }
        None => Ok(None),
    }
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
        assert_eq!(tool.description, Some("A sample tool".to_string()));
        assert_eq!(tool.input_schema, json!({}));
        let annotations = tool.annotations.as_ref().unwrap();
        assert_eq!(annotations.title, Some("My Tool".to_string()));
        assert_eq!(annotations.read_only_hint, false);
        assert_eq!(annotations.destructive_hint, false);
        assert_eq!(annotations.idempotent_hint, true);
        assert_eq!(annotations.open_world_hint, false);
    }

    #[test]
    fn test_tool_details_deserialization_empty_annotations() {
        let json_data = json!({
            "name": "my_tool",
            "namespaced_name": "acute-partridge.my_tool",
            "description": null,
            "input_schema": {},
            "annotations": {}
        });

        let tool_details: ToolDetails = serde_json::from_value(json_data).unwrap();
        assert_eq!(tool_details.name, "my_tool");
        assert_eq!(tool_details.namespaced_name, "acute-partridge.my_tool");
        assert_eq!(tool_details.description, None);
        assert_eq!(tool_details.input_schema, json!({}));
        assert_eq!(tool_details.annotations, None);
    }

    #[test]
    fn test_tool_details_deserialization_null_annotations() {
        let json_data = json!({
            "name": "my_tool",
            "namespaced_name": "acute-partridge.my_tool",
            "description": null,
            "input_schema": {},
            "annotations": null
        });

        let tool_details: ToolDetails = serde_json::from_value(json_data).unwrap();
        assert_eq!(tool_details.name, "my_tool");
        assert_eq!(tool_details.namespaced_name, "acute-partridge.my_tool");
        assert_eq!(tool_details.description, None);
        assert_eq!(tool_details.input_schema, json!({}));
        assert_eq!(tool_details.annotations, None);
    }
}
