use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VirtualFolderReference {
    /// The kubernetes resource name of the virtual folder
    pub name: String,
    /// The kubernetes namespace the folder is defined in, if different from the namespace
    /// of this resource.
    pub namespace: Option<String>,
    /// The path to use inside the virtual folder.
    pub virtual_path: String,
    /// Quota as size in bytes. 0 means unlimited, -1 means included in user quota. Please note
    /// that quota is updated if files are added/removed via SFTPGo otherwise a quota scan or a
    /// manual quota update is needed
    pub quota_size: Option<i64>,
    /// Quota as number of files. 0 means unlimited, , -1 means included in user quota. Please
    /// note that quota is updated if files are added/removed via SFTPGo otherwise a quota scan
    /// or a manual quota update is needed
    pub quota_files: Option<i32>,
}
