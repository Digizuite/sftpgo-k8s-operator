use crate::client::AuthorizedSftpgoClientBase;
use crate::error_response::{handle_response, Result};
use crate::GenericResponseBody;
use async_trait::async_trait;
use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::collections::HashMap;

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Eq, Debug, Clone, Default)]
#[repr(u8)]
pub enum UserStatus {
    #[default]
    Disabled = 0,
    Enabled = 1,
}

// #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
// pub struct VirtualFolder {
//     pub name: String,
//     pub mapped_path: String,
//     pub description: Option<String>,
//
// }

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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct UserRequest {
    pub status: UserStatus,
    pub username: String,
    pub email: Option<String>,
    pub description: Option<String>,
    pub expiration_date: Option<i64>,
    pub password: Option<String>,
    pub public_keys: Option<Vec<String>>,
    pub home_dir: String,
    pub uid: Option<i32>,
    pub gid: Option<i32>,
    pub max_sessions: Option<i32>,
    pub quota_size: Option<i64>,
    pub quota_files: Option<i32>,
    pub permissions: HashMap<String, Vec<String>>,
    pub upload_bandwidth: Option<i64>,
    pub download_bandwidth: Option<i64>,
    pub upload_data_transfer: Option<i64>,
    pub download_data_transfer: Option<i64>,
    pub total_data_transfer: Option<i64>,
    pub filesystem: FileSystem,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct UserResponse {
    pub id: i32,
    pub status: UserStatus,
    pub username: String,
    pub password: Option<String>,
    pub email: Option<String>,
    pub description: Option<String>,
    pub expiration_date: Option<i64>,
    pub public_keys: Option<Vec<String>>,
    pub home_dir: String,
    pub uid: Option<i32>,
    pub gid: Option<i32>,
    pub max_sessions: Option<i32>,
    pub quota_size: Option<i64>,
    pub quota_files: Option<i32>,
    pub permissions: Option<HashMap<String, Vec<String>>>,
    pub upload_bandwidth: Option<i64>,
    pub download_bandwidth: Option<i64>,
    pub upload_data_transfer: Option<i64>,
    pub download_data_transfer: Option<i64>,
    pub total_data_transfer: Option<i64>,
}

#[async_trait]
pub trait UsersClient: Send + Sync {
    async fn add_user(&self, user: UserRequest) -> Result<UserResponse>;
    async fn update_user(&self, user: UserRequest) -> Result<GenericResponseBody>;
    async fn get_user(&self, username: &str) -> Result<Option<UserResponse>>;
    async fn delete_user(&self, username: &str) -> Result<()>;
}

#[async_trait]
impl<Client> UsersClient for Client
where
    Client: AuthorizedSftpgoClientBase + Sync + Send,
{
    async fn add_user(&self, user: UserRequest) -> Result<UserResponse> {
        let url = self.url_for("/api/v2/users")?;

        let auth_header_value = self.get_auth_context().get_auth_header_value().await?;
        let res = self
            .get_client()
            .post(url)
            .header(AUTHORIZATION, auth_header_value)
            .json(&user)
            .send()
            .await?;

        handle_response(res).await
    }
    async fn update_user(&self, user: UserRequest) -> Result<GenericResponseBody> {
        let url = self.url_for(&format!("/api/v2/users/{}", user.username))?;

        let auth_header_value = self.get_auth_context().get_auth_header_value().await?;
        let res = self
            .get_client()
            .put(url)
            .header(AUTHORIZATION, auth_header_value)
            .json(&user)
            .send()
            .await?;

        handle_response(res).await
    }
    async fn get_user(&self, username: &str) -> Result<Option<UserResponse>> {
        let url = self.url_for(&format!("/api/v2/users/{}", username))?;

        let auth_header_value = self.get_auth_context().get_auth_header_value().await?;
        let res = self
            .get_client()
            .get(url)
            .header(AUTHORIZATION, auth_header_value)
            .send()
            .await?;

        if res.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        handle_response(res).await
    }
    async fn delete_user(&self, username: &str) -> Result<()> {
        let url = self.url_for(&format!("/api/v2/users/{}", username))?;

        let auth_header_value = self.get_auth_context().get_auth_header_value().await?;
        let res = self
            .get_client()
            .delete(url)
            .header(AUTHORIZATION, auth_header_value)
            .send()
            .await?;

        if res.status() == StatusCode::NOT_FOUND || res.status() == StatusCode::OK {
            return Ok(());
        }

        handle_response(res).await
    }
}
