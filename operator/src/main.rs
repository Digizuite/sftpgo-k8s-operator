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
use crds::{SftpgoFolder, SftpgoUser};
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::{Secret, Service};
use kube::client::Client;
use kube::runtime::watcher;
use kube::Api;
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

    reconcilers.spawn(make_reconciler(
        kubernetes_client.clone(),
        sftpgo_api_resource_reconciler::<SftpgoUser>,
        |c| c,
    ));
    reconcilers.spawn(make_reconciler(
        kubernetes_client.clone(),
        sftpgo_api_resource_reconciler::<SftpgoFolder>,
        |c| c,
    ));

    info!("Reconcilers spawned");

    while let Some(res) = reconcilers.join_next().await {
        res?;
    }

    Ok(())
}
