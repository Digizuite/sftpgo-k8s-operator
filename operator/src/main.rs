mod admin_reconciler;
mod consts;
mod filesystem;
mod finalizers;
mod folder_reconciler;
mod reconciler;
mod sftpgo_multi_client;
mod sftpgo_server_reconciler;
mod user_reconciler;
mod viper_environment_serializer;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

pub use crate::reconciler::Error;
use crate::reconciler::{make_reconciler, sftpgo_api_resource_reconciler, ContextData};
use crate::sftpgo_server_reconciler::reconcile_sftpgo_server;
use crds::{SftpgoAdmin, SftpgoFolder, SftpgoUser};
use futures::{stream, Stream, StreamExt, TryStreamExt};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Secret, Service};
use kube::client::Client;
use kube::runtime::watcher;
use kube::{Api, ResourceExt};
use kube_runtime::reflector::ObjectRef;
use kube_runtime::WatchStreamExt;
use tokio::task::{JoinError, JoinSet};

pub fn default<T: Default>() -> T {
    Default::default()
}

#[tokio::main]
async fn main() -> Result<(), JoinError> {
    pretty_env_logger::init_timed();

    info!("Starting SFTPGo Operator");

    let kubernetes_client = Client::try_default()
        .await
        .expect("Expected a valid KUBECONFIG environment variable.");

    let mut reconcilers = JoinSet::new();

    let deployments_api: Api<Deployment> = Api::all(kubernetes_client.clone());
    let secrets_api: Api<Secret> = Api::all(kubernetes_client.clone());
    let services_api: Api<Service> = Api::all(kubernetes_client.clone());

    reconcilers.spawn(make_reconciler(
        kubernetes_client.clone(),
        reconcile_sftpgo_server,
        |c| {
            let watcher_config =
                watcher::Config::default().labels("managed-by=sftpgo-server-operator");
            c.owns(deployments_api, watcher_config.clone())
                .owns(secrets_api, watcher_config.clone())
                .owns(services_api, watcher_config)
        },
    ));

    let user_folder_trigger = watch_users_for_folder(kubernetes_client.clone());

    reconcilers.spawn(make_reconciler(
        kubernetes_client.clone(),
        sftpgo_api_resource_reconciler::<SftpgoUser>,
        |c| c.watches_stream(user_folder_trigger, map_user),
    ));
    reconcilers.spawn(make_reconciler(
        kubernetes_client.clone(),
        sftpgo_api_resource_reconciler::<SftpgoFolder>,
        |c| c,
    ));
    reconcilers.spawn(make_reconciler(
        kubernetes_client.clone(),
        sftpgo_api_resource_reconciler::<SftpgoAdmin>,
        |c| c,
    ));

    info!("Reconcilers spawned");

    while let Some(res) = reconcilers.join_next().await {
        res?;
    }

    Ok(())
}

fn map_user(u: SftpgoUser) -> Option<ObjectRef<SftpgoUser>> {
    Some(ObjectRef::from_obj(&u))
}

fn watch_users_for_folder(
    kubernetes_client: Client,
) -> impl Stream<Item = Result<SftpgoUser, watcher::Error>> + Send + Sized + 'static {
    let folders_api: Api<SftpgoFolder> = Api::all(kubernetes_client);

    watcher(folders_api, default())
        .applied_objects()
        .and_then(list_users)
        .flat_map_unordered(None, |users| {
            stream::iter(users.into_iter().flatten().map(Ok))
        })
}

async fn list_users(folder: SftpgoFolder) -> Result<Vec<SftpgoUser>, watcher::Error> {
    info!("Getting user list");
    let kubernetes_client = Client::try_default()
        .await
        .expect("Expected a valid KUBECONFIG environment variable.");

    let api: Api<SftpgoUser> = Api::all(kubernetes_client);
    let response = api.list(&default()).await.map_err(|e| {
        watcher::Error::WatchError(kube::core::ErrorResponse {
            code: 0,
            message: e.to_string(),
            reason: "".to_string(),
            status: "".to_string(),
        })
    })?;

    let folder_name = folder.name_any();
    let folder_namespace = &folder.metadata.namespace.unwrap_or("".to_string());

    let items = response
        .items
        .into_iter()
        .filter(|u| {
            if let Some(user_ns) = &u.metadata.namespace {
                u.spec
                    .configuration
                    .virtual_folders
                    .as_ref()
                    .is_some_and(|ve| {
                        ve.iter().any(|v| {
                            if v.name != folder_name {
                                return false;
                            }

                            if let Some(ns) = &v.namespace {
                                ns == folder_namespace
                            } else {
                                user_ns == folder_namespace
                            }
                        })
                    })
            } else {
                false
            }
        })
        .collect();

    Ok(items)
}
