mod finalizers;
mod reconciler;
mod sftpgo_server_reconciler;
mod viper_environment_serializer;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use crate::reconciler::{ContextData, make_reconciler};
use crate::sftpgo_server_reconciler::reconcile_sftpgo_server;
use kube::client::Client;

pub fn default<T: Default>() -> T {
    Default::default()
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init_timed();

    let kubernetes_client = Client::try_default()
        .await
        .expect("Expected a valid KUBECONFIG environment variable.");

    let reconcilers = vec![make_reconciler(
        kubernetes_client.clone(),
        reconcile_sftpgo_server,
    )];

    futures::future::join_all(reconcilers).await;
}
