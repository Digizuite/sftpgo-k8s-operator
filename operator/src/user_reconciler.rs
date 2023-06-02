use crate::filesystem::calculate_file_system;
use crate::reconciler::{ContextData, Error};
use crate::sftpgo_multi_client::get_api_client;
use crate::{default, finalizers};
use crds::{
    SftpgoUser, SftpgoUserConfiguration, SftpgoUserResourceStatus, SftpgoUserStatus, UserPermission,
};
use kube::api::Patch;
use kube::runtime::controller::Action;
use kube::{Api, ResourceExt};
use sftpgo_client::{UserRequest, UserStatus};
use std::collections::HashMap;
use std::sync::Arc;

pub async fn reconcile_user(
    resource: Arc<SftpgoUser>,
    context: Arc<ContextData>,
) -> Result<Action, Error> {
    info!("Running user conciliation");

    let name = resource.name_any();

    let namespace = resource.namespace().ok_or(Error::UserInput(
        "Expected SftpgoUser resource to be namespaced. Can't deploy to unknown namespace."
            .to_string(),
    ))?;

    let sftpgo_user_api: Api<SftpgoUser> =
        Api::namespaced(context.kubernetes_client.clone(), &namespace);
    let mut resource = sftpgo_user_api.get(&name).await?;

    let user_configuration = resource.spec.configuration.clone();
    let server_ref = resource.spec.server_reference.clone();

    let api_client = get_api_client(&server_ref, &context, &namespace).await?;

    if resource.metadata.deletion_timestamp.is_some() {
        if let Some(status) = &resource.status {
            api_client.delete(&status.last_username).await?;
        }

        api_client.delete(&user_configuration.username).await?;

        finalizers::remove_finalizer::<SftpgoUser>(
            context.kubernetes_client.clone(),
            &name,
            &namespace,
        )
        .await?;

        return Ok(Action::await_change());
    }

    resource = finalizers::ensure_finalizer(resource, context.kubernetes_client.clone()).await?;

    if let Some(status) = &resource.status {
        if status.last_username != user_configuration.username {
            info!(
                "Username changed from {} to {}. Deleting old user since usernames cannot be changed",
                status.last_username, user_configuration.username
            );

            api_client.delete(&status.last_username).await?;

            let mut copy = resource.clone();
            copy.status = Some(SftpgoUserResourceStatus {
                last_username: user_configuration.username.clone(),
                ..default()
            });

            resource = sftpgo_user_api
                .patch_status(&name, &default(), &Patch::Merge(copy))
                .await?;
        } else {
            info!("Username not changed");
        }
    } else {
        info!("No status set");

        let mut copy = resource.clone();
        copy.status = Some(SftpgoUserResourceStatus {
            last_username: user_configuration.username.clone(),
            ..default()
        });

        resource = sftpgo_user_api
            .patch_status(&name, &default(), &Patch::Merge(copy))
            .await?;
    }

    let permissions = calculate_permissions(&user_configuration);

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

    if api_client
        .get(&user_configuration.username)
        .await?
        .is_some()
    {
        info!("User already exists");

        api_client.update(&user_request).await?;
        info!("User updated");
    } else {
        info!("User does not exist, creating");

        let created_user = api_client.create(&user_request).await?;

        info!("User created");

        let mut copy = resource.clone();
        copy.status = Some(SftpgoUserResourceStatus {
            last_username: user_configuration.username.clone(),
            user_id: Some(created_user.id),
        });

        sftpgo_user_api
            .patch_status(&name, &default(), &Patch::Merge(copy))
            .await?;

        info!("Resource status updated")
    }

    Ok(Action::await_change())
}

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
