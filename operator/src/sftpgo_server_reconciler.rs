use crate::reconciler::Error;
use crate::{finalizers, ContextData};
use crds::SftpgoServer;
use kube::runtime::controller::Action;
use kube::ResourceExt;
use std::sync::Arc;
use std::time::Duration;

pub async fn reconcile_sftpgo_server(
    resource: Arc<SftpgoServer>,
    context: Arc<ContextData>,
) -> Result<Action, Error> {
    let client = context.client.clone();

    let namespace = resource.namespace().ok_or(Error::UserInputError(
        "Expected SftpgoServer resource to be namespaced. Can't deploy to unknown namespace.",
    ))?;

    let name = resource.name_any();

    info!("Reconciling {namespace}/{name}");

    if resource.metadata.deletion_timestamp.is_some() {
        debug!("Resource {namespace}/{name} is marked for deletion, removing finalizer");
        finalizers::remove_finalizer::<SftpgoServer>(client.clone(), &name, &namespace).await?;
        debug!("Finalizer removed from {namespace}/{name}");
        return Ok(Action::await_change());
    }

    if resource
        .metadata
        .finalizers
        .as_ref()
        .map_or(true, |finalizers| finalizers.is_empty())
    {
        debug!("Finalizer not found on resource {namespace}/{name}, adding");
        finalizers::add_finalizer::<SftpgoServer>(client.clone(), &name, &namespace).await?;
        debug!("Finalizer added to {namespace}/{name}")
    } else {
        debug!("Finalizer found on resource {namespace}/{name}");
    }

    Ok(Action::requeue(Duration::from_secs(15)))
}
