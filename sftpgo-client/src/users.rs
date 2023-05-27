use crate::auth::{AuthContext};
use crate::client::SftpgoClientBase;
use crate::error_response::{handle_response, Result};
use async_trait::async_trait;
use reqwest::header::AUTHORIZATION;
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
pub struct AddUserRequest {
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
    pub permissions: Option<HashMap<String, Vec<String>>>,
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
    pub password: String,
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
pub trait UsersClient: SftpgoClientBase {
    async fn add_user(
        &self,
        user: AddUserRequest,
        auth_context: &dyn AuthContext,
    ) -> Result<UserResponse> {
        let url = self.url_for("/api/v2/users")?;

        let auth_header_value = auth_context.get_auth_header_value().await?;
        let res = self
            .get_client()
            .post(url)
            .header(AUTHORIZATION, auth_header_value)
            .json(&user)
            .send()
            .await?;

        handle_response(res).await
    }
    async fn get_user(
        &self,
        username: &str,
        auth_context: &dyn AuthContext,
    ) -> Result<UserResponse> {
        let url = self.url_for(&format!("/api/v2/users/{}", username))?;

        let auth_header_value = auth_context.get_auth_header_value().await?;
        let res = self
            .get_client()
            .get(url)
            .header(AUTHORIZATION, auth_header_value)
            .send()
            .await?;

        handle_response(res).await
    }
}

impl<T> UsersClient for T where T: SftpgoClientBase {}
