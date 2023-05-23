mod finalizers;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use crds::Server;
use futures::future::err;
use futures::stream::StreamExt;
use kube::runtime::watcher::Config;
use kube::Resource;
use kube::ResourceExt;
use kube::{client::Client, runtime::controller::Action, runtime::Controller, Api};
use std::fmt::Debug;
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    pretty_env_logger::init_timed();

    let kubernetes_client = Client::try_default()
        .await
        .expect("Expected a valid KUBECONFIG environment variable.");

    let crd_api = Api::all(kubernetes_client.clone());
    let context = Arc::new(ContextData {
        client: kubernetes_client.clone(),
    });

    Controller::new(crd_api.clone(), Config::default())
        .run(reconcile, error_policy, context)
        .for_each(|res| async move {
            match res {
                Ok(o) => debug!("reconciled: {:?}", o),
                Err(e) => error!("reconcile failed: {}", e),
            }
        })
        .await;
}

pub struct ContextData {
    pub client: Client,
}

async fn reconcile(resource: Arc<Server>, context: Arc<ContextData>) -> Result<Action, Error> {
    let client = context.client.clone();

    let namespace = resource.namespace().ok_or(Error::UserInputError(
        "Expected Server resource to be namespaced. Can't deploy to unknown namespace.",
    ))?;

    let name = resource.name_any();

    info!("Reconciling {namespace}/{name}");

    if resource
        .metadata
        .finalizers
        .as_ref()
        .map_or(true, |finalizers| finalizers.is_empty())
    {
        debug!("Finalizer not found on resource {namespace}/{name}, adding");
        finalizers::add_finalizer::<Server>(client.clone(), &name, &namespace).await?;
        debug!("Finalizer added to {namespace}/{name}")
    } else {
        debug!("Finalizer found on resource {namespace}/{name}");
    }

    Ok(Action::requeue(Duration::from_secs(15)))
}

fn error_policy<TResource: Debug>(
    echo: Arc<TResource>,
    error: &Error,
    _context: Arc<ContextData>,
) -> Action {
    error!("Reconciliation error:\n{:?}.\n{:?}", error, echo);
    Action::requeue(Duration::from_secs(15))
}

/// All errors possible to occur during reconciliation
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Any error originating from the `kube-rs` crate
    #[error("Kubernetes reported error: {source}")]
    KubeError {
        #[from]
        source: kube::Error,
    },

    /// Error in user input or Echo resource definition, typically missing fields.
    #[error("Invalid CRD: {0}")]
    UserInputError(&'static str),
}
