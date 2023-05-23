use k8s_openapi::NamespaceResourceScope;
use kube::api::{Patch, PatchParams};
use kube::{Api, Client, Error, Resource};
use serde::de::DeserializeOwned;
use serde_json::json;
use std::fmt::Debug;

pub async fn add_finalizer<TResource>(
    client: Client,
    name: &str,
    namespace: &str,
) -> Result<TResource, Error>
where
    TResource: Resource<Scope = NamespaceResourceScope> + Clone + DeserializeOwned + Debug,
    <TResource as Resource>::DynamicType: Default,
{
    debug!("Adding finalizer to {namespace}/{name}");
    let api = Api::namespaced(client, namespace);

    let finalizer = json!({
        "metadata": {
            "finalizers": ["sftpgo.zlepper.dk/finalizer"]
        }
    });

    let patch = Patch::Merge(&finalizer);
    api.patch(name, &PatchParams::default(), &patch).await
}

pub async fn delete_finalizer<TResource>(
    client: Client,
    name: &str,
    namespace: &str,
) -> Result<TResource, Error>
where
    TResource: Resource<Scope = NamespaceResourceScope> + Clone + DeserializeOwned + Debug,
    <TResource as Resource>::DynamicType: Default,
{
    debug!("Deleting finalizer from {namespace}/{name}");
    let api = Api::namespaced(client, namespace);
    let finalizer = json!({
        "metadata": {
            "finalizers": null
        }
    });

    let patch = Patch::Merge(&finalizer);
    api.patch(name, &PatchParams::default(), &patch).await
}
