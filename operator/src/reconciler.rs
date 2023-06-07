use crate::default;
use crate::finalizers::{ensure_finalizer, remove_finalizer};
use crate::sftpgo_multi_client::{get_api_client, SftpgoMultiClient};
use crate::viper_environment_serializer::ViperEnvironmentSerializerError;
use async_trait::async_trait;
use crds::{ServerReference, SftpgoStatus};
use futures::stream::StreamExt;
use futures::TryFuture;
use k8s_openapi::NamespaceResourceScope;
use kube::api::Patch;
use kube::core::object::HasStatus;
use kube::runtime::controller::Action;
use kube::runtime::watcher::Config;
use kube::runtime::Controller;
use kube::{Api, Client, CustomResourceExt, Resource, ResourceExt};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sftpgo_client::{
    AuthorizedSftpgoClient, CreatedFrom, Creates, EasyRestSftpgoClient, Existing,
    RefreshableAdminAuthContext, SftpgoClient,
};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

pub async fn make_reconciler<TResource, ReconcilerFut, ReconcilerFn, CustomizeFn>(
    kubernetes_client: Client,
    recon: ReconcilerFn,
    customize_controller: CustomizeFn,
) where
    TResource:
        Clone + Resource + CustomResourceExt + DeserializeOwned + Debug + Send + Sync + 'static,
    TResource::DynamicType: Debug + Unpin + Eq + Hash + Clone + Default,
    ReconcilerFn: FnMut(Arc<TResource>, Arc<ContextData>) -> ReconcilerFut,
    ReconcilerFut: TryFuture<Ok = Action, Error = Error> + Send + 'static,
    CustomizeFn: FnOnce(Controller<TResource>) -> Controller<TResource>,
{
    info!("Starting reconciler for {:#?}", TResource::crd_name());

    let crd_api = Api::all(kubernetes_client.clone());
    let context = Arc::new(ContextData {
        kubernetes_client: kubernetes_client.clone(),
        sftpgo_client: SftpgoMultiClient::new(),
    });

    let mut controller_setup: Controller<TResource> =
        Controller::new(crd_api.clone(), Config::default());

    controller_setup = customize_controller(controller_setup);

    controller_setup
        .run(recon, error_policy, context)
        .for_each(|res| async move {
            match res {
                Ok(o) => debug!("reconciled: {:?}", o),
                Err(e) => error!("reconcile failed: {:?}", e),
            }
        })
        .await
}

fn error_policy<TResource>(
    echo: Arc<TResource>,
    error: &Error,
    _context: Arc<ContextData>,
) -> Action
where
    TResource:
        Clone + Resource + CustomResourceExt + DeserializeOwned + Debug + Send + Sync + 'static,
{
    error!(
        "Reconciliation error while reconciling type {}:\n{:?}.\n{:?}",
        TResource::crd_name(),
        error,
        echo
    );
    Action::requeue(Duration::from_secs(15))
}

pub struct ContextData {
    pub kubernetes_client: Client,
    pub sftpgo_client: SftpgoMultiClient,
}

/// All errors possible to occur during reconciliation
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Any error originating from the `kube-rs` crate
    #[error("Kubernetes reported error: {source}")]
    Kube {
        #[from]
        source: kube::Error,
    },

    #[error("Kubernetes watcher error: {source}")]
    KubeWatcher {
        #[from]
        source: kube_runtime::watcher::Error,
    },

    #[error("Viper serializer reported error: {source}")]
    ViperSerializer {
        #[from]
        source: ViperEnvironmentSerializerError,
    },

    /// Error in user input or resource definition, typically missing fields.
    #[error("Invalid CRD: {0}")]
    UserInput(String),

    #[error("The requested resource {0} was found, but is not yet ready.")]
    NotReady(String),

    #[error("Sftpgo client reported error: {0}")]
    SftpgoRequestFailed(#[from] sftpgo_client::SftpgoError),

    #[error("Error while decoding base64: {0}")]
    DecodeError(#[from] base64::DecodeError),
}

#[async_trait]
pub trait SftpgoResource {
    type Request: Serialize + Sync + Creates<Self::Response>;
    type Response: for<'de> Deserialize<'de> + CreatedFrom<Self::Request>;

    fn get_name(&self) -> &str;

    async fn get_request(
        &self,
        context: &ContextData,
        namespace: &String,
    ) -> Result<Self::Request, Error>;

    fn get_server_reference(&self) -> &ServerReference;
}

pub async fn sftpgo_api_resource_reconciler<TCrd>(
    resource: Arc<TCrd>,
    context: Arc<ContextData>,
) -> Result<Action, Error>
where
    TCrd: SftpgoResource
        + Clone
        + Resource<Scope = NamespaceResourceScope>
        + HasStatus
        + CustomResourceExt
        + DeserializeOwned
        + Serialize
        + Debug
        + Send
        + Sync
        + 'static,
    TCrd::DynamicType: Debug + Unpin + Eq + Hash + Clone + Default,
    <TCrd as HasStatus>::Status: SftpgoStatus + Default,
    AuthorizedSftpgoClient<RefreshableAdminAuthContext<SftpgoClient>>:
        EasyRestSftpgoClient<TCrd::Request, TCrd::Response>,
{
    let name = resource.name_any();

    let namespace = resource.namespace().ok_or(Error::UserInput(
        "Expected SftpgoUser resource to be namespaced. Can't deploy to unknown namespace."
            .to_string(),
    ))?;

    let resource_api: Api<TCrd> = Api::namespaced(context.kubernetes_client.clone(), &namespace);

    let mut resource = resource_api.get(&name).await?;
    let server_ref = resource.get_server_reference();

    let api_client = get_api_client(server_ref, &context, &namespace).await?;

    let sftpgo_name = resource.get_name().to_string();
    if resource.meta().deletion_timestamp.is_some() {
        info!("Resource {} is being deleted, cleaning up", sftpgo_name);
        if let Some(status) = &resource.status() {
            api_client.delete(status.get_last_name()).await?;
            info!("Deleted old name {} from SFTPGo", status.get_last_name());
        }

        api_client.delete(&sftpgo_name).await?;

        info!("Deleted {} from SFTPGo", sftpgo_name);

        remove_finalizer::<TCrd>(context.kubernetes_client.clone(), &name, &namespace).await?;

        info!("Removed finalizer");

        return Ok(Action::await_change());
    }

    resource = ensure_finalizer(resource, context.kubernetes_client.clone()).await?;

    if let Some(ref mut status) = &mut resource.status_mut() {
        if status.get_last_name() != sftpgo_name {
            info!(
                "Name changed from {} to {}, deleting old resource",
                status.get_last_name(),
                sftpgo_name
            );

            api_client.delete(status.get_last_name()).await?;

            status.set_last_name(&sftpgo_name);

            resource = resource_api
                .patch_status(&name, &default(), &Patch::Merge(resource))
                .await?;
        } else {
            info!("Name did not change, no need to delete old resource");
        }
    } else {
        info!("No status set");

        let status = resource.status_mut();

        let mut s = TCrd::Status::default();
        s.set_last_name(&sftpgo_name);

        *status = Some(s);

        resource = resource_api
            .patch_status(&name, &default(), &Patch::Merge(resource))
            .await?;
    }

    let request = resource.get_request(&context, &namespace).await?;

    if api_client.get(&sftpgo_name).await?.is_some() {
        info!("Updating resource {}", sftpgo_name);

        api_client.update(&request).await?;
        info!("Updated resource {}", sftpgo_name);
    } else {
        info!("Creating resource {}", sftpgo_name);

        let created_resource = api_client.create(&request).await?;

        info!("Created resource {}", sftpgo_name);

        let status = resource.status_mut();

        let mut s = TCrd::Status::default();
        s.set_last_name(&sftpgo_name);
        s.set_id(Some(created_resource.id()));

        *status = Some(s);

        resource_api
            .patch_status(&name, &default(), &Patch::Merge(resource))
            .await?;

        info!("Updated status for resource {}", sftpgo_name);
    }

    Ok(Action::await_change())
}
