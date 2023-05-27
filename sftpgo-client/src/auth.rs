use crate::error_response::Result;
use async_trait::async_trait;
use base64::engine::general_purpose;
use base64::Engine;

pub fn create_basic_auth_header(username: &str, password: &str) -> String {
    let encoded = general_purpose::STANDARD.encode(format!("{}:{}", username, password));
    format!("Basic {}", encoded,)
}

pub fn create_bearer_auth_header(token: &str) -> String {
    format!("Bearer {}", token)
}

#[async_trait]
pub trait AuthContext: Sync {
    async fn get_auth_header_value(&self) -> Result<String>;
}
