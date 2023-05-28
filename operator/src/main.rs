mod finalizers;
mod reconciler;
mod sftpgo_multi_client;
mod sftpgo_server_reconciler;
mod user_reconciler;
mod viper_environment_serializer;
mod consts;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use crate::reconciler::{make_reconciler, ContextData};
use crate::sftpgo_server_reconciler::reconcile_sftpgo_server;
use kube::client::Client;
use tokio::task::{JoinError, JoinSet};

pub fn default<T: Default>() -> T {
    Default::default()
}

#[tokio::main]
async fn main() -> Result<(), JoinError> {
    pretty_env_logger::init_timed();

    let kubernetes_client = Client::try_default()
        .await
        .expect("Expected a valid KUBECONFIG environment variable.");

    let mut reconcilers = JoinSet::new();

    reconcilers.spawn(make_reconciler(
        kubernetes_client.clone(),
        reconcile_sftpgo_server,
    ));
    reconcilers.spawn(make_reconciler(
        kubernetes_client.clone(),
        user_reconciler::reconcile_user,
    ));

    while let Some(res) = reconcilers.join_next().await {
        res?;
    }

    Ok(())
}
