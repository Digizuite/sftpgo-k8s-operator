use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
