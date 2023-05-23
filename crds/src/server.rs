use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[kube(
    group = "sftpgo.zlepper.dk",
    version = "v1alpha1",
    kind = "Server",
    plural = "servers",
    derive = "PartialEq",
    namespaced
)]
pub struct ServerSpec {
    pub replicas: u32,
}
