use crate::{FileSystem, ServerReference};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Defines the filesystem for the virtual folder and the used quota limits. The same folder can be
/// shared among multiple users and each user can have different quota limits or a different
/// virtual path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SftpgoFolderConfiguration {
    /// unique name for this virtual folder
    pub name: String,
    /// absolute filesystem path to use as virtual folder
    pub mapped_path: String,
    /// optional description
    pub description: Option<String>,
    /// Storage filesystem details
    pub filesystem: FileSystem,
}

#[derive(CustomResource, Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[kube(
    group = "sftpgo.zlepper.dk",
    version = "v1alpha1",
    kind = "SftpgoFolder",
    plural = "sftpgofolders",
    derive = "PartialEq",
    status = "SftpgoFolderResourceStatus",
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct SftpgoFolderSpec {
    pub configuration: SftpgoFolderConfiguration,
    #[serde(rename = "sftpgoServerReference")]
    pub server_reference: ServerReference,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema, Default)]
pub struct SftpgoFolderResourceStatus {
    last_name: String,
}
