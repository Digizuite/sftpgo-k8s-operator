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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct UserRequest {
    pub status: UserStatus,
    pub username: String,
    pub email: Option<String>,
    pub description: Option<String>,
    pub expiration_date: Option<i64>,
    pub password: String,
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
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct UserResponse {
    pub id: i32,
    pub status: UserStatus,
    pub username: String,
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
