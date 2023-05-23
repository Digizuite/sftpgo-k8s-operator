mod consts;
mod finalizers;

use kube::runtime::watcher::Config;
use kube::Resource;
use kube::ResourceExt;
use kube::{client::Client, runtime::controller::Action, runtime::Controller, Api};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    let kubernetes_client = Client::try_default()
        .await
        .expect("Expected a valid KUBECONFIG environment variable.");
}
