use crate::consts::GROUP;
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
    let api = Api::namespaced(client, namespace);

    let finalizer = json!({
        "metadata": {
            "finalizers": [format!("{}/finalizer", GROUP)]
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
    let api = Api::namespaced(client, namespace);
    let finalizer = json!({
        "metadata": {
            "finalizers": null
        }
    });

    let patch = Patch::Merge(&finalizer);
    api.patch(name, &PatchParams::default(), &patch).await
}
