use crate::client::SftpgoClientBase;
use crate::filesystem::FileSystem;
use crate::rest_client::{EasyRestSftpgoClient, Named};
use crate::{Creates, Existing, Result};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct FolderRequest {
    pub name: String,
    pub mapped_path: Option<String>,
    pub description: Option<String>,
    pub filesystem: FileSystem,
}

impl Named for FolderRequest {
    fn name(&self) -> &str {
        &self.name
    }
}

impl Creates<FolderResponse> for FolderRequest {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct FolderResponse {
    pub id: i32,
    pub name: String,
    pub mapped_path: Option<String>,
    pub description: Option<String>,
    pub filesystem: FileSystem,
}

impl Existing for FolderResponse {
    fn name(&self) -> &str {
        &self.name
    }

    fn id(&self) -> i32 {
        self.id
    }
}

impl<Client> EasyRestSftpgoClient<FolderRequest, FolderResponse> for Client
where
    Client: SftpgoClientBase,
{
    fn get_url(&self, path: Option<&str>) -> Result<Url> {
        if let Some(path) = path {
            self.url_for(&format!("/api/v2/folders/{}", path))
        } else {
            self.url_for("/api/v2/folders")
        }
    }
}
