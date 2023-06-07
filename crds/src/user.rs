use crate::sftpgo_server_reference::ServerReference;
use crate::{FileSystem, SftpgoStatus};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum SftpgoUserStatus {
    Disabled,
    Enabled,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UserPermission {
    #[default]
    All,
    List,
    Download,
    Upload,
    Overwrite,
    CreateDirs,
    Rename,
    RenameFiles,
    RenameDirs,
    Delete,
    DeleteFiles,
    DeleteDirs,
    CreateSymlinks,
    Chmod,
    Chown,
    Chtimes,
}

impl ToString for UserPermission {
    fn to_string(&self) -> String {
        match self {
            UserPermission::All => "*".to_string(),
            UserPermission::List => "list".to_string(),
            UserPermission::Download => "download".to_string(),
            UserPermission::Upload => "upload".to_string(),
            UserPermission::Overwrite => "overwrite".to_string(),
            UserPermission::CreateDirs => "create_dirs".to_string(),
            UserPermission::Rename => "rename".to_string(),
            UserPermission::RenameFiles => "rename_files".to_string(),
            UserPermission::RenameDirs => "rename_dirs".to_string(),
            UserPermission::Delete => "delete".to_string(),
            UserPermission::DeleteFiles => "delete_files".to_string(),
            UserPermission::DeleteDirs => "delete_dirs".to_string(),
            UserPermission::CreateSymlinks => "create_symlinks".to_string(),
            UserPermission::Chmod => "chmod".to_string(),
            UserPermission::Chown => "chown".to_string(),
            UserPermission::Chtimes => "chtimes".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DirectoryPermission {
    pub path: String,
    pub permissions: Vec<UserPermission>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SftpgoUserConfiguration {
    /// The username of the user
    pub username: String,
    /// Password of the user. Changes to this field will not propagate to the user after creation as we have
    /// no way of retrieving the password from the server.
    pub password: String,
    pub enabled: Option<SftpgoUserStatus>,
    pub global_permissions: Vec<UserPermission>,
    pub per_directory_permissions: Option<Vec<DirectoryPermission>>,
    pub filesystem: FileSystem,
    pub home_dir: String,
}

#[derive(CustomResource, Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[kube(
    group = "sftpgo.zlepper.dk",
    version = "v1alpha1",
    kind = "SftpgoUser",
    plural = "sftpgousers",
    derive = "PartialEq",
    status = "SftpgoUserResourceStatus",
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct SftpgoUserSpec {
    pub configuration: SftpgoUserConfiguration,
    /// This way you force the user to login again, if connected, and so to use the new configuration
    pub disconnect_on_change: Option<bool>,
    #[serde(rename = "sftpgoServerReference")]
    pub server_reference: ServerReference,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema, Default)]
pub struct SftpgoUserResourceStatus {
    pub last_username: String,
    pub user_id: Option<i32>,
}

impl SftpgoStatus for SftpgoUserResourceStatus {
    fn get_last_name(&self) -> &str {
        &self.last_username
    }

    fn set_last_name(&mut self, name: &str) {
        self.last_username = name.to_string();
    }

    fn get_id(&self) -> Option<i32> {
        self.user_id
    }

    fn set_id(&mut self, id: Option<i32>) {
        self.user_id = id;
    }
}