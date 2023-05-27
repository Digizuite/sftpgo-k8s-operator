use crate::sftpgo_server_reference::ServerReference;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum SftpgoUserStatus {
    Disabled,
    Enabled,
}

#[derive(CustomResource, Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[kube(
    group = "sftpgo.zlepper.dk",
    version = "v1alpha1",
    kind = "SftpgoUser",
    plural = "sftpgousers",
    derive = "PartialEq",
    namespaced
)]
pub struct SftpgoUserSpec {
    pub username: String,
    pub password: String,
    pub status: Option<SftpgoUserStatus>,
    #[serde(rename = "sftpgoServerReference")]
    pub server_reference: ServerReference,
}
