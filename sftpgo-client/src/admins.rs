use crate::{
    AuthorizedSftpgoClientBase, Creates, EasyRestSftpgoClient, Existing, Named, UserStatus,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct AdminRequest {
    pub username: String,
    pub description: Option<String>,
    pub password: String,
    pub email: Option<String>,
    pub permissions: Vec<String>,
    pub status: UserStatus,
    pub role: Option<String>,
}

impl Named for AdminRequest {
    fn name(&self) -> &str {
        &self.username
    }
}

impl Creates<AdminResponse> for AdminRequest {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct AdminResponse {
    pub id: i32,
    pub username: String,
    pub status: UserStatus,
    pub description: Option<String>,
    pub email: Option<String>,
    pub permissions: Vec<String>,
    pub role: Option<String>,
}

impl Existing for AdminResponse {
    fn name(&self) -> &str {
        &self.username
    }

    fn id(&self) -> i32 {
        self.id
    }
}

impl<Client> EasyRestSftpgoClient<AdminRequest, AdminResponse> for Client
where
    Client: AuthorizedSftpgoClientBase + Send + Sync,
{
    fn get_url(&self, path: Option<&str>) -> crate::Result<Url> {
        if let Some(path) = path {
            self.url_for(&format!("/api/v2/admins/{}", path))
        } else {
            self.url_for("/api/v2/admins")
        }
    }
}
