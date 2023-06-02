use crate::reconciler::{ContextData, Error};
use crate::sftpgo_multi_client::get_api_client;
use crate::{default, finalizers};
use crds::{
    AzureBlobStorageAccessTier, AzureBlobStorageAuthorization, FileSystem, SftpgoUser,
    SftpgoUserConfiguration, SftpgoUserResourceStatus, SftpgoUserStatus, UserPermission,
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

    if resource.metadata.deletion_timestamp.is_some() {
        let api_client = get_api_client(&server_ref, &context, &namespace).await?;

        if let Some(status) = &resource.status {
            api_client.delete_user(&status.last_username).await?;
        }

        api_client.delete_user(&user_configuration.username).await?;

        finalizers::remove_finalizer::<SftpgoUser>(
            context.kubernetes_client.clone(),
            &name,
            &namespace,
        )
        .await?;

        return Ok(Action::await_change());
    }

    if resource
        .metadata
        .finalizers
        .as_ref()
        .map_or(true, |finalizers| finalizers.is_empty())
    {
        debug!("Finalizer not found on resource {namespace}/{name}, adding");
        resource = finalizers::add_finalizer::<SftpgoUser>(
            context.kubernetes_client.clone(),
            &name,
            &namespace,
        )
        .await?;
        debug!("Finalizer added to {namespace}/{name}")
    } else {
        debug!("Finalizer found on resource {namespace}/{name}");
    }

    if let Some(status) = &resource.status {
        if status.last_username != user_configuration.username {
            info!(
                "Username changed from {} to {}. Deleting old user since usernames cannot be changed",
                status.last_username, user_configuration.username
            );

            let api_client = get_api_client(&server_ref, &context, &namespace).await?;

            api_client.delete_user(&status.last_username).await?;

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

    let api_client = get_api_client(&server_ref, &context, &namespace).await?;

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
        filesystem: calculate_file_system(&user_configuration).await?,
        ..default()
    };

    if api_client
        .get_user(&user_configuration.username)
        .await?
        .is_some()
    {
        info!("User already exists");

        api_client.update_user(user_request).await?;
        info!("User updated");
    } else {
        info!("User does not exist, creating");

        let created_user = api_client.add_user(user_request).await?;

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

async fn calculate_file_system(
    user_configuration: &SftpgoUserConfiguration,
) -> Result<sftpgo_client::users::FileSystem, Error> {
    let fs = match &user_configuration.filesystem {
        FileSystem::Local {
            read_buffer_size,
            write_buffer_size,
        } => sftpgo_client::users::FileSystem {
            provider: sftpgo_client::users::FileSystemProvider::LocalFilesystem,
            config: sftpgo_client::users::FileSystemConfig::OsConfig {
                read_buffer_size: read_buffer_size.unwrap_or(0),
                write_buffer_size: write_buffer_size.unwrap_or(0),
            },
        },
        FileSystem::AzureBlobStorage(blob) => sftpgo_client::users::FileSystem {
            provider: sftpgo_client::users::FileSystemProvider::AzureBlobStorage,
            config: sftpgo_client::users::FileSystemConfig::AzureBlobStorage(Box::new(
                sftpgo_client::users::FileSystemConfigAzureBlobStorage {
                    auth: match &blob.authorization {
                        AzureBlobStorageAuthorization::SharedKey{ account_key, account_name, container}  => sftpgo_client::users::FileSystemConfigAzureBlobStorageAuthorization::SharedKey {
                            account_name: account_name.clone(),
                            container: container.clone(),
                            account_key: sftpgo_client::users::SftpgoSecret {
                                status: sftpgo_client::users::SftpgoSecretStatus::Plain,
                                payload: account_key.clone(),
                                ..default()
                            },
                        },
                        AzureBlobStorageAuthorization::SharedAccessSignatureUrl(url) => sftpgo_client::users::FileSystemConfigAzureBlobStorageAuthorization::SharedAccessSignatureUrl {
                            sas_url: sftpgo_client::users::SftpgoSecret {
                                status: sftpgo_client::users::SftpgoSecretStatus::Plain,
                                payload: url.clone(),
                                ..default()
                            }
                        }
                    },
                    endpoint: blob.endpoint.clone(),
                    upload_part_size: blob.upload_part_size.unwrap_or(5),
                    upload_concurrency: blob.upload_concurrency.unwrap_or(5),
                    download_part_size: blob.download_part_size.unwrap_or(5),
                    download_concurrency: blob.download_concurrency.unwrap_or(5),
                    access_tier: blob.access_tier.map(|t| match t {
                        AzureBlobStorageAccessTier::Hot => sftpgo_client::users::FileSystemConfigAzureBlobStorageAccessTier::Hot,
                        AzureBlobStorageAccessTier::Cool => sftpgo_client::users::FileSystemConfigAzureBlobStorageAccessTier::Cool,
                        AzureBlobStorageAccessTier::Archive => sftpgo_client::users::FileSystemConfigAzureBlobStorageAccessTier::Archive,
                    }),
                    key_prefix: blob.key_prefix.clone(),
                    use_emulator: blob.use_emulator,
                },
            )),
        },
    };

    Ok(fs)
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
