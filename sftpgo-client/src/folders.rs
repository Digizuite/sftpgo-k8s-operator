use crate::client::SftpgoClientBase;
use crate::filesystem::FileSystem;
use crate::rest_client::{EasyRestSftpgoClient, Named};
use crate::Result;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct FolderRequest {
    name: String,
    mapped_path: String,
    description: Option<String>,
    filesystem: FileSystem,
}

impl Named for FolderRequest {
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct FolderResponse {
    id: i32,
    name: String,
    mapped_path: String,
    description: Option<String>,
    filesystem: FileSystem,
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
