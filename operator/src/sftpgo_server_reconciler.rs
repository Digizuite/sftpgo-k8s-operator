use crate::reconciler::Error;
use crate::viper_environment_serializer::ViperEnvironmentSerializer;
use crate::{default, finalizers, ContextData};
use crds::{SftpgoServer, SftpgoServerSpec};
use futures::StreamExt;
use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{
    Container, ContainerPort, EnvVar, PodSpec, PodTemplateSpec, Service, ServicePort, ServiceSpec,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::runtime::controller::Action;
use kube::{Api, Client, ResourceExt};
use serde::Serialize;
use std::collections::BTreeMap;
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
        debug!("Resource {namespace}/{name} is marked for deletion");

        return if delete_deployment(&name, &namespace, client.clone()).await? {
            debug!("Deployment for resource {namespace}/{name} deleted. Removing finalizer");

            finalizers::remove_finalizer::<SftpgoServer>(client.clone(), &name, &namespace).await?;
            debug!("Finalizer removed from {namespace}/{name}");
            Ok(Action::await_change())
        } else {
            debug!("Queued deletion of deployment for resource {namespace}/{name}");
            Ok(Action::requeue(Duration::from_secs(15)))
        };
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

    deploy(&resource.spec, &name, &namespace, client).await?;

    Ok(Action::requeue(Duration::from_secs(15)))
}

const DEFAULT_IMAGE: &str = "drakkan/sftpgo:v2.5";

async fn deploy(
    resource: &SftpgoServerSpec,
    name: &str,
    namespace: &str,
    client: Client,
) -> Result<(), Error> {
    let image = resource.image.as_deref().unwrap_or(DEFAULT_IMAGE);

    let deployment_name = format!("{}-deployment", name);

    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), namespace);
    let service_api: Api<Service> = Api::namespaced(client, namespace);

    let mut labels = resource.labels.clone().unwrap_or_default();
    labels.insert("app".to_string(), name.to_string());

    let mut annotations = BTreeMap::new();
    annotations.insert("managed_by".to_string(), "sftpgo-operator".to_string());
    annotations.insert("managed_by_resource".to_string(), name.to_string());

    let mut env_serializer = ViperEnvironmentSerializer::new_with_prefix("SFTPGO_".to_string());
    resource.configuration.serialize(&mut env_serializer)?;

    let mut expected_ports = vec![];

    if let Some(resource_configuration) = &resource.configuration {
        if let Some(httpd_config) = &resource_configuration.httpd {
            if let Some(httpd_bindings) = &httpd_config.bindings {
                for http_biding in httpd_bindings {
                    let port_number = http_biding.port.unwrap_or(8080);
                    let port_name = format!("http-{}", port_number);
                    expected_ports.push(ContainerPort {
                        name: Some(port_name),
                        container_port: port_number,
                        ..default()
                    });
                }
            }
        }
    }

    let expected_container = Container {
        name: "sftpgo".to_string(),
        image: Some(image.to_string()),
        env: Some(
            env_serializer
                .values
                .into_iter()
                .map(|p| EnvVar {
                    name: p.key,
                    value: Some(p.value),
                    value_from: None,
                })
                .collect(),
        ),
        ports: Some(expected_ports.clone()),
        ..default()
    };
    let expected_pod_spec = PodSpec {
        containers: vec![expected_container.clone()],
        ..default()
    };
    let expected_deployment = Deployment {
        metadata: ObjectMeta {
            name: Some(deployment_name.clone()),
            namespace: Some(namespace.to_string()),
            labels: Some(labels.clone()),
            annotations: Some(annotations),
            ..default()
        },
        spec: Some(DeploymentSpec {
            replicas: resource.replicas,
            selector: LabelSelector {
                match_labels: Some(labels.clone()),
                ..default()
            },
            template: PodTemplateSpec {
                metadata: Some(ObjectMeta {
                    labels: Some(labels.clone()),
                    ..default()
                }),
                spec: Some(expected_pod_spec.clone()),
            },
            ..default()
        }),
        ..default()
    };

    let expected_service_ports: Vec<ServicePort> = expected_ports
        .iter()
        .map(|p| ServicePort {
            name: p.name.clone(),
            protocol: p.protocol.clone(),
            port: p.container_port,
            target_port: p.name.as_ref().map(|n| IntOrString::String(n.clone())),
            ..default()
        })
        .collect();

    let expected_service = Service {
        metadata: ObjectMeta {
            name: Some(name.to_string()),
            namespace: Some(namespace.to_string()),
            ..default()
        },
        spec: Some(ServiceSpec {
            selector: Some(labels.clone()),
            ports: Some(expected_service_ports.clone()),
            ..default()
        }),
        ..default()
    };

    if let Some(existing) = deployments_api.get_opt(&deployment_name).await? {
        debug!("Deployment {namespace}/{deployment_name} already exists");

        let mut copy = existing.clone();

        if let Some(ref mut spec) = &mut copy.spec {
            if resource.replicas.is_some() {
                spec.replicas = resource.replicas;
            }

            if let Some(ref mut pod_spec) = &mut spec.template.spec {
                if pod_spec.containers.is_empty() {
                    pod_spec.containers = expected_pod_spec.containers;
                } else {
                    let container = &mut pod_spec.containers[0];

                    if container.image.as_deref() != Some(image) {
                        container.image = Some(image.to_string());
                    }

                    if container.env != expected_container.env {
                        container.env = expected_container.env;
                    }

                    if container.ports != expected_container.ports {
                        container.ports = expected_container.ports;
                    }
                }
            } else {
                spec.template.spec = Some(expected_pod_spec.clone());
            }
        } else {
            copy.spec = expected_deployment.spec;
        }

        if copy != existing {
            debug!("Deployment {namespace}/{deployment_name} has changed, updating");
            deployments_api
                .replace(&deployment_name, &default(), &copy)
                .await?;
            debug!("Deployment {namespace}/{deployment_name} updated")
        } else {
            debug!("Deployment {namespace}/{deployment_name} has not changed, skipping")
        }
    } else {
        debug!("Deployment {namespace}/{deployment_name} does not exist, creating");

        deployments_api
            .create(&default(), &expected_deployment)
            .await?;
    }

    if let Some(existing) = service_api.get_opt(name).await? {
        debug!("Service {namespace}/{name} already exists");

        let mut copy = existing.clone();

        if let Some(ref mut spec) = &mut copy.spec {
            if let Some(ref mut ports) = &mut spec.ports {
                if ports != &expected_service_ports {
                    *ports = expected_service_ports;
                }
            } else {
                spec.ports = Some(expected_service_ports);
            }
        } else {
            copy.spec = expected_service.spec;
        }

        if copy != existing {
            debug!("Service {namespace}/{name} has changed, updating");
            service_api.replace(name, &default(), &copy).await?;
            debug!("Service {namespace}/{name} updated")
        } else {
            debug!("Service {namespace}/{name} has not changed, skipping")
        }
    } else {
        debug!("Service {namespace}/{name} does not exist, creating");

        service_api.create(&default(), &expected_service).await?;

        debug!("Service {namespace}/{name} created")
    }

    Ok(())
}

async fn delete_deployment(name: &str, namespace: &str, client: Client) -> Result<bool, Error> {
    let deployment_name = format!("{}-deployment", name);

    let deployments_api: Api<Deployment> = Api::namespaced(client, namespace);

    let result = deployments_api.delete(&deployment_name, &default()).await?;

    Ok(result.is_right())
}
