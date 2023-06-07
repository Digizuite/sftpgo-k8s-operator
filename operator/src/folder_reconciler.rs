use crate::filesystem::calculate_file_system;
use crate::reconciler::{ContextData, SftpgoResource};
use crate::Error;
use async_trait::async_trait;
use crds::{ServerReference, SftpgoFolder};
use sftpgo_client::folders::{FolderRequest, FolderResponse};

#[async_trait]
impl SftpgoResource for SftpgoFolder {
    type Request = FolderRequest;
    type Response = FolderResponse;

    fn get_name(&self) -> &str {
        &self.spec.configuration.name
    }

    async fn get_request(
        &self,
        _context: &ContextData,
        _namespace: &String,
    ) -> Result<Self::Request, Error> {
        let folder_configuration = &self.spec.configuration;

        let request = FolderRequest {
            name: folder_configuration.name.clone(),
            description: folder_configuration.description.clone(),
            mapped_path: folder_configuration.mapped_path.clone(),
            filesystem: calculate_file_system(Some(&folder_configuration.filesystem)).await?,
        };

        Ok(request)
    }

    fn get_server_reference(&self) -> &ServerReference {
        &self.spec.server_reference
    }
}
