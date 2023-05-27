mod admin_token;
mod auth;
mod client;
mod error_response;
mod users;

pub use admin_token::{AdminAccessToken, AdminAccessTokenClient};
pub use client::SftpgoClient;

mod prelude {
    pub(crate) use crate::admin_token::AdminAccessTokenClient;
    pub(crate) use crate::client::SftpgoClient;
}
