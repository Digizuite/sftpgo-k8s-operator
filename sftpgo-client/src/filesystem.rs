use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Debug, Clone, Default)]
#[repr(u8)]
pub enum FileSystemProvider {
    #[default]
    LocalFilesystem = 0,
    S3 = 1,
    GoogleCloudStorage = 2,
    AzureBlobStorage = 3,
    LocalFileSystemEncrypted = 4,
    Sftp = 5,
    Http = 6,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub enum SftpgoSecretStatus {
    #[default]
    Plain,
    Aes256Gcm,
    Secretbox,
    GCP,
    AWS,
    ValueTransit,
    AzureKeyVault,
    Redacted,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct SftpgoSecret {
    pub status: SftpgoSecretStatus,
    pub payload: String,
    pub key: Option<String>,
    pub additional_data: Option<String>,
    pub mode: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum FileSystemConfigAzureBlobStorageAccessTier {
    Hot,
    Cool,
    Archive,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum FileSystemConfigAzureBlobStorageAuthorization {
    SharedKey {
        account_name: String,
        container: String,
        account_key: SftpgoSecret,
    },
    SharedAccessSignatureUrl {
        sas_url: SftpgoSecret,
    },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct FileSystemConfigAzureBlobStorage {
    #[serde(flatten)]
    pub auth: FileSystemConfigAzureBlobStorageAuthorization,
    pub endpoint: Option<String>,
    pub upload_part_size: i32,
    pub upload_concurrency: i32,
    pub download_part_size: i32,
    pub download_concurrency: i32,
    pub access_tier: Option<FileSystemConfigAzureBlobStorageAccessTier>,
    pub key_prefix: Option<String>,
    pub use_emulator: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum FileSystemConfig {
    #[serde(rename = "osconfig")]
    OsConfig {
        read_buffer_size: i32,
        write_buffer_size: i32,
    },
    #[serde(rename = "azblobconfig")]
    AzureBlobStorage(Box<FileSystemConfigAzureBlobStorage>),
}

impl Default for FileSystemConfig {
    fn default() -> Self {
        FileSystemConfig::OsConfig {
            write_buffer_size: 0,
            read_buffer_size: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct FileSystem {
    pub provider: FileSystemProvider,
    #[serde(flatten)]
    pub config: FileSystemConfig,
}
