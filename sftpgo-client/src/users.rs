use crate::error_response::Result;
use crate::filesystem::FileSystem;
use crate::rest_client::{Creates, EasyRestSftpgoClient, Named};
use crate::AuthorizedSftpgoClientBase;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::collections::HashMap;
use url::Url;

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

impl Named for UserRequest {
    fn name(&self) -> &str {
        &self.username
    }
}

impl Creates<UserResponse> for UserRequest {}

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

impl<Client> EasyRestSftpgoClient<UserRequest, UserResponse> for Client
where
    Client: AuthorizedSftpgoClientBase + Send + Sync,
{
    fn get_url(&self, path: Option<&str>) -> Result<Url> {
        if let Some(path) = path {
            self.url_for(&format!("/api/v2/users/{}", path))
        } else {
            self.url_for("/api/v2/users")
        }
    }
}
