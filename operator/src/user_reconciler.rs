use crate::default;
use crate::filesystem::calculate_file_system;
use crate::reconciler::{ContextData, Error, SftpgoResource};
use async_trait::async_trait;
use crds::{
    ServerReference, SftpgoFolder, SftpgoStatus, SftpgoUser, SftpgoUserConfiguration,
    SftpgoUserStatus, UserPermission,
};
use kube::Api;
use sftpgo_client::{UserRequest, UserResponse, UserStatus};
use std::collections::HashMap;

fn calculate_permissions(
    user_configuration: &SftpgoUserConfiguration,
) -> HashMap<String, Vec<String>> {
    let mut permissions = HashMap::<String, Vec<String>>::new();
    let mut root_permissions: Vec<String> = user_configuration
        .global_permissions
        .iter()
        .map(|permission| permission.to_string())
        .collect();

    if root_permissions.is_empty() {
        root_permissions.push(UserPermission::All.to_string());
    }

    permissions.insert("/".to_string(), root_permissions);

    if let Some(dir_permissions) = &user_configuration.per_directory_permissions {
        for dir_permission in dir_permissions.iter() {
            let path_permissions = dir_permission
                .permissions
                .iter()
                .map(|permission| permission.to_string())
                .collect();

            permissions.insert(dir_permission.path.clone(), path_permissions);
        }
    }
    permissions
}

async fn get_virtual_folder_reference(
    namespace: &String,
    crd_ref: &crds::VirtualFolderReference,
    context: &ContextData,
) -> Result<sftpgo_client::virtual_folder_reference::VirtualFolderReference, Error> {
    let ns = crd_ref.namespace.as_ref().unwrap_or(namespace);

    let folder_client: Api<SftpgoFolder> = Api::namespaced(context.kubernetes_client.clone(), ns);

    if let Some(folder) = folder_client.get_opt(&crd_ref.name).await? {
        if let Some(status) = folder.status {
            if status.get_id().is_none() {
                return Err(Error::NotReady(crd_ref.name.to_string()));
            }

            let name = status.get_last_name();

            let folder_ref = sftpgo_client::virtual_folder_reference::VirtualFolderReference {
                name: name.to_string(),
                virtual_path: crd_ref.virtual_path.clone(),
                quota_size: crd_ref.quota_size.unwrap_or(0),
                quota_files: crd_ref.quota_files.unwrap_or(0),
            };

            Ok(folder_ref)
        } else {
            Err(Error::NotReady(crd_ref.name.to_string()))
        }
    } else {
        Err(Error::UserInput(format!(
            "Virtual folder {} not found in namespace {}",
            crd_ref.name, ns
        )))
    }
}

pub trait MapEnabled<To> {
    fn map_enabled(&self) -> To;
}

impl MapEnabled<UserStatus> for SftpgoUserStatus {
    fn map_enabled(&self) -> UserStatus {
        match self {
            SftpgoUserStatus::Disabled => UserStatus::Disabled,
            SftpgoUserStatus::Enabled => UserStatus::Enabled,
        }
    }
}

#[async_trait]
impl SftpgoResource for SftpgoUser {
    type Request = UserRequest;
    type Response = UserResponse;

    fn get_name(&self) -> &str {
        &self.spec.configuration.username
    }

    async fn get_request(
        &self,
        context: &ContextData,
        namespace: &String,
    ) -> Result<Self::Request, Error> {
        let user_configuration = &self.spec.configuration;

        let permissions = calculate_permissions(user_configuration);

        let virtual_folders = if let Some(folders) = &self.spec.configuration.virtual_folders {
            let mut references = Vec::new();

            for folder in folders {
                let reference = get_virtual_folder_reference(namespace, folder, context).await?;
                references.push(reference);
            }

            Some(references)
        } else {
            None
        };

        let user_request = UserRequest {
            username: user_configuration.username.clone(),
            password: Some(user_configuration.password.clone()),
            status: user_configuration
                .enabled
                .map_or(UserStatus::Enabled, |status| status.map_enabled()),
            permissions: permissions.clone(),
            home_dir: user_configuration.home_dir.clone(),
            filesystem: calculate_file_system(user_configuration.filesystem.as_ref()).await?,
            virtual_folders,
            ..default()
        };

        Ok(user_request)
    }

    fn get_server_reference(&self) -> &ServerReference {
        &self.spec.server_reference
    }
}
