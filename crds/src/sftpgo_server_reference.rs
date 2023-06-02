use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConnectionSecret {
    pub name: String,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConnectionOverride {
    pub url: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ServerReference {
    pub name: Option<String>,
    pub namespace: Option<String>,
    pub connection_secret: Option<ConnectionSecret>,
    pub override_values: Option<ConnectionOverride>,
}
