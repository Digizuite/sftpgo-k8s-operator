use crate::default;
use crate::filesystem::calculate_file_system;
use crate::reconciler::{Error, SftpgoResource};
use async_trait::async_trait;
use crds::{
    ServerReference, SftpgoStatus, SftpgoUser, SftpgoUserConfiguration, SftpgoUserResourceStatus,
    SftpgoUserStatus, UserPermission,
};
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

#[async_trait]
impl SftpgoResource for SftpgoUser {
    type Status = SftpgoUserResourceStatus;
    type Request = UserRequest;
    type Response = UserResponse;

    fn get_name(&self) -> &str {
        &self.spec.configuration.username
    }

    async fn get_request(&self) -> Result<Self::Request, Error> {
        let user_configuration = &self.spec.configuration;

        let permissions = calculate_permissions(user_configuration);

        let user_request = UserRequest {
            username: user_configuration.username.clone(),
            password: Some(user_configuration.password.clone()),
            status: user_configuration
                .enabled
                .map_or(UserStatus::Enabled, |status| match status {
                    SftpgoUserStatus::Disabled => UserStatus::Disabled,
                    SftpgoUserStatus::Enabled => UserStatus::Enabled,
                }),
            permissions: permissions.clone(),
            home_dir: user_configuration.home_dir.clone(),
            filesystem: calculate_file_system(&user_configuration.filesystem).await?,
            ..default()
        };

        Ok(user_request)
    }

    fn get_server_reference(&self) -> &ServerReference {
        &self.spec.server_reference
    }

    fn get_status(&self) -> &Option<Self::Status> {
        &self.status
    }

    fn set_last_name(&mut self, name: &str) {
        if let Some(ref mut status) = self.status {
            status.set_last_name(name);
        } else {
            let mut status = SftpgoUserResourceStatus::default();
            status.set_last_name(name);
            self.status = Some(status);
        }
    }

    fn set_id(&mut self, id: Option<i32>) {
        if let Some(ref mut status) = self.status {
            status.set_id(id);
        } else {
            let mut status = SftpgoUserResourceStatus::default();
            status.set_id(id);
            self.status = Some(status);
        }
    }
}
