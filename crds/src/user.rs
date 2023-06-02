use crate::sftpgo_server_reference::ServerReference;
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
#[serde(rename_all = "camelCase")]
pub enum ValueFrom {
    #[serde(rename_all = "camelCase")]
    ConfigMapKeyRef { name: String, key: String },
    #[serde(rename_all = "camelCase")]
    SecretKeyRef { name: String, key: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ProvidedValue<T> {
    Value(T),
    ValueFrom(ValueFrom),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AzureBlobStorageAuthorization {
    #[serde(rename_all = "camelCase")]
    SharedKey {
        /// The name of the container to use. Sftpgo does not create this automatically, so make sure
        /// it exists before using it here.
        container: String,
        account_name: String,
        account_key: String,
    },
    SharedAccessSignatureUrl(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum AzureBlobStorageAccessTier {
    Hot,
    Cool,
    Archive,
}

impl ToString for AzureBlobStorageAccessTier {
    fn to_string(&self) -> String {
        match self {
            AzureBlobStorageAccessTier::Hot => "hot".to_string(),
            AzureBlobStorageAccessTier::Cool => "cool".to_string(),
            AzureBlobStorageAccessTier::Archive => "archive".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileSystemAzureBlobStorage {
    pub authorization: AzureBlobStorageAuthorization,
    /// optional endpoint. Default is "blob.core.windows.net". If you use the emulator the
    /// endpoint must include the protocol, for example "http://127.0.0.1:10000"
    pub endpoint: Option<String>,
    /// the buffer size (in MB) to use for multipart uploads. If this value is not set, the
    /// default value (5MB) will be used.
    pub upload_part_size: Option<i32>,
    /// the number of parts to upload in parallel. If this value is not set, the default value
    /// (5) will be used
    pub upload_concurrency: Option<i32>,
    /// the buffer size (in MB) to use for multipart downloads. If this value is not set, the
    /// default value (5MB) will be used.
    pub download_part_size: Option<i32>,
    /// the number of parts to download in parallel. If this value is not set, the default
    /// value (5) will be used
    pub download_concurrency: Option<i32>,
    pub access_tier: Option<AzureBlobStorageAccessTier>,

    /// key_prefix is similar to a chroot directory for a local filesystem. If specified the
    /// user will only see contents that starts with this prefix and so you can restrict access
    /// to a specific virtual folder. The prefix, if not empty, must not start with "/" and must
    /// end with "/". If empty the whole container contents will be available
    pub key_prefix: Option<String>,

    pub use_emulator: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum FileSystem {
    #[serde(rename_all = "camelCase")]
    Local {
        read_buffer_size: Option<i32>,
        write_buffer_size: Option<i32>,
    },
    #[serde(rename_all = "camelCase")]
    AzureBlobStorage(Box<FileSystemAzureBlobStorage>),
}

impl Default for FileSystem {
    fn default() -> Self {
        FileSystem::Local {
            write_buffer_size: Some(0),
            read_buffer_size: Some(0),
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
