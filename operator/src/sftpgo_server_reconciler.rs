use crate::consts::{SECRET_KEY_PASSWORD, SECRET_KEY_URL, SECRET_KEY_USERNAME};
use crate::reconciler::Error;
use crate::viper_environment_serializer::ViperEnvironmentSerializer;
use crate::{default, ContextData};
use crds::{SftpgoServer, SftpgoServerSpec};
use k8s_openapi::api::apps::v1::{Deployment, DeploymentSpec};
use k8s_openapi::api::core::v1::{
    Container, ContainerPort, EnvVar, EnvVarSource, PodSpec, PodTemplateSpec, Secret,
    SecretKeySelector, Service, ServicePort, ServiceSpec,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{LabelSelector, ObjectMeta, OwnerReference};
use k8s_openapi::apimachinery::pkg::util::intstr::IntOrString;
use kube::api::{Patch, PatchParams};
use kube::runtime::controller::Action;
use kube::{Api, Client, Resource, ResourceExt};
use rand::distributions::{Alphanumeric, DistString};
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;
use std::vec::Vec;

pub async fn reconcile_sftpgo_server(
    resource: Arc<SftpgoServer>,
    context: Arc<ContextData>,
) -> Result<Action, Error> {
    let controller =
        DeploymentController::new(resource.as_ref(), context.kubernetes_client.clone())?;

    let namespace = resource.namespace().ok_or(Error::UserInput(
        "Expected SftpgoServer resource to be namespaced. Can't deploy to unknown namespace."
            .to_string(),
    ))?;

    let name = resource.name_any();

    info!("Reconciling {namespace}/{name}");

    if resource.metadata.deletion_timestamp.is_some() {
        debug!("Resource {namespace}/{name} is marked for deletion");
        return Ok(Action::await_change());
    }

    controller.ensure_secret().await?;
    controller.ensure_service().await?;
    controller.ensure_deployment().await?;

    Ok(Action::requeue(Duration::from_secs(3600)))
}

const DEFAULT_IMAGE: &str = "drakkan/sftpgo:v2.5";

struct DeploymentController {
    name: String,
    namespace: String,
    kubernetes_client: Client,
    owner_reference: OwnerReference,
    resource: SftpgoServerSpec,
}

impl DeploymentController {
    fn new(resource: &SftpgoServer, client: Client) -> Result<DeploymentController, Error> {
        let namespace = resource.namespace().ok_or(Error::UserInput(
            "Expected SftpgoServer resource to be namespaced. Can't deploy to unknown namespace."
                .to_string(),
        ))?;

        let name = resource.name_any();

        // Always comes from the api, so according to the docs it's safe to unwrap
        let owner_reference = resource.controller_owner_ref(&()).unwrap();

        Ok(DeploymentController {
            name,
            namespace,
            kubernetes_client: client,
            owner_reference,
            resource: resource.spec.clone(),
        })
    }

    fn get_labels(&self) -> BTreeMap<String, String> {
        let mut labels = self.resource.labels.clone().unwrap_or_default();
        labels.insert("app".to_string(), self.name.to_string());
        labels.insert(
            "managed-by".to_string(),
            "sftpgo-server-operator".to_string(),
        );

        labels
    }

    fn get_expected_ports(&self) -> Vec<ContainerPort> {
        let mut expected_ports = vec![];

        if let Some(httpd_bindings) = self
            .resource
            .configuration
            .as_ref()
            .and_then(|c| c.httpd.as_ref())
            .and_then(|h| h.bindings.as_ref())
            .and_then(|b| if b.is_empty() { None } else { Some(b) })
        {
            for http_biding in httpd_bindings {
                let port_number = http_biding.port.unwrap_or(8080);
                let port_name = format!("http-{}", port_number);
                expected_ports.push(ContainerPort {
                    name: Some(port_name),
                    container_port: port_number,
                    ..default()
                });
            }
        } else {
            expected_ports.push(ContainerPort {
                name: Some("http-8080".to_string()),
                container_port: 8080,
                ..default()
            });
        }

        if let Some(sftpd_bindings) = self
            .resource
            .configuration
            .as_ref()
            .and_then(|c| c.sftpd.as_ref())
            .and_then(|s| s.bindings.as_ref())
        {
            for sftpd_binding in sftpd_bindings {
                let port_number = sftpd_binding.port.unwrap_or(2022);
                let port_name = format!("sftp-{}", port_number);
                expected_ports.push(ContainerPort {
                    name: Some(port_name),
                    container_port: port_number,
                    ..default()
                });
            }
        }

        if let Some(ftpd) = self
            .resource
            .configuration
            .as_ref()
            .and_then(|c| c.ftpd.as_ref())
        {
            if let Some(bindings) = &ftpd.bindings {
                for binding in bindings {
                    let port_number = binding.port.unwrap_or(21);
                    let port_name = format!("ftp-{}", port_number);
                    expected_ports.push(ContainerPort {
                        name: Some(port_name),
                        container_port: port_number,
                        ..default()
                    });
                }
            }

            if let Some(passive_port_range) = &ftpd.passive_port_range {
                for port_number in passive_port_range.start..=passive_port_range.end {
                    let port_name = format!("ftp-data-{}", port_number);
                    expected_ports.push(ContainerPort {
                        name: Some(port_name),
                        container_port: port_number,
                        ..default()
                    });
                }
            }
        }

        expected_ports
    }

    fn get_deployment_name(&self) -> String {
        format!("{}-deployment", self.name)
    }

    fn get_admin_user_secret_name(&self) -> String {
        format!("{}-admin-user", self.name)
    }

    async fn ensure_service(&self) -> Result<(), Error> {
        let name = &self.name;
        let namespace = &self.namespace;
        let client = self.kubernetes_client.clone();

        let expected_service_ports: Vec<ServicePort> = self
            .get_expected_ports()
            .iter()
            .map(|p| ServicePort {
                name: p.name.clone(),
                protocol: p.protocol.clone().or(Some("TCP".to_string())),
                port: p.container_port,
                target_port: p.name.as_ref().map(|n| IntOrString::String(n.clone())),
                ..default()
            })
            .collect();

        let expected_service = Service {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(namespace.to_string()),
                owner_references: Some(vec![self.owner_reference.clone()]),
                labels: Some(self.get_labels()),
                ..default()
            },
            spec: Some(ServiceSpec {
                selector: Some(self.get_labels()),
                ports: Some(expected_service_ports.clone()),
                ..default()
            }),
            ..default()
        };

        let service_api: Api<Service> = Api::namespaced(client, namespace);

        let serverside = PatchParams::apply("sftpgo-operator").force();
        service_api
            .patch(name, &serverside, &Patch::Apply(expected_service))
            .await?;
        Ok(())
    }

    async fn ensure_deployment(&self) -> Result<(), Error> {
        let mut env_serializer = ViperEnvironmentSerializer::new_with_prefix("SFTPGO_".to_string());
        self.resource.configuration.serialize(&mut env_serializer)?;

        let mut configuration_variables: Vec<EnvVar> = env_serializer
            .values
            .into_iter()
            .map(|p| EnvVar {
                name: p.key,
                value: Some(p.value),
                ..default()
            })
            .collect();

        configuration_variables.push(EnvVar {
            name: "SFTPGO_DATA_PROVIDER__CREATE_DEFAULT_ADMIN".to_string(),
            value: Some("true".to_string()),
            ..default()
        });

        configuration_variables.push(EnvVar {
            name: "SFTPGO_DEFAULT_ADMIN_USERNAME".to_string(),
            value_from: Some(EnvVarSource {
                secret_key_ref: Some(SecretKeySelector {
                    name: Some(self.get_admin_user_secret_name()),
                    key: "username".to_string(),
                    ..default()
                }),
                ..default()
            }),
            ..default()
        });

        configuration_variables.push(EnvVar {
            name: "SFTPGO_DEFAULT_ADMIN_PASSWORD".to_string(),
            value_from: Some(EnvVarSource {
                secret_key_ref: Some(SecretKeySelector {
                    name: Some(self.get_admin_user_secret_name()),
                    key: "password".to_string(),
                    ..default()
                }),
                ..default()
            }),
            ..default()
        });

        let expected_ports = self.get_expected_ports();
        let namespace = &self.namespace;
        let labels = self.get_labels();

        let image = &self.resource.image.as_deref().unwrap_or(DEFAULT_IMAGE);
        let expected_container = Container {
            name: "sftpgo".to_string(),
            image: Some(image.to_string()),
            env: Some(configuration_variables),
            ports: Some(expected_ports.clone()),
            ..default()
        };
        let expected_pod_spec = PodSpec {
            containers: vec![expected_container.clone()],
            node_selector: self.resource.node_selector.clone(),
            ..default()
        };
        let deployment_name = self.get_deployment_name();
        let expected_deployment = Deployment {
            metadata: ObjectMeta {
                name: Some(deployment_name.clone()),
                namespace: Some(namespace.to_string()),
                labels: Some(labels.clone()),
                owner_references: Some(vec![self.owner_reference.clone()]),
                ..default()
            },
            spec: Some(DeploymentSpec {
                replicas: self.resource.replicas,
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

        let deployments_api: Api<Deployment> =
            Api::namespaced(self.kubernetes_client.clone(), namespace);
        let serverside = PatchParams::apply("sftpgo-operator").force();

        deployments_api
            .patch(
                &deployment_name,
                &serverside,
                &Patch::Apply(expected_deployment),
            )
            .await?;
        Ok(())
    }

    async fn ensure_secret(&self) -> Result<String, Error> {
        let name = &self.name;
        let namespace = &self.namespace;

        let secret_api: Api<Secret> = Api::namespaced(self.kubernetes_client.clone(), namespace);

        let http_binding = self
            .resource
            .configuration
            .as_ref()
            .and_then(|c| c.httpd.as_ref())
            .and_then(|h| h.bindings.as_ref())
            .and_then(|b| b.first());

        let http_port = http_binding.and_then(|b| b.port).unwrap_or(8080);

        let http_protocol = if http_binding.and_then(|b| b.enable_https).unwrap_or(false) {
            "https"
        } else {
            "http"
        };

        let management_url = format!("{http_protocol}://{name}.{namespace}.svc:{http_port}");

        // Intentionally not using the patch api as we cannot ensure we generate the same password
        // every time, and that would cause issues with actually accessing sftpgo.
        let admin_user_secret_name = format!("{}-admin-user", name);
        if let Some(ref mut existing) = secret_api.get_opt(&admin_user_secret_name).await? {
            debug!("Secret {} already exists", admin_user_secret_name);

            let mut changed = false;
            if let Some(ref mut sd) = existing.string_data {
                let entry = sd.get_mut("url");
                if let Some(e) = entry {
                    if *e != management_url {
                        *e = management_url;
                        changed = true;
                    }
                } else {
                    sd.insert(SECRET_KEY_URL.to_string(), management_url);
                    changed = true;
                }
            }

            if changed {
                debug!("Updating secret {}", admin_user_secret_name);
                secret_api
                    .replace(&admin_user_secret_name, &default(), existing)
                    .await?;
            }
        } else {
            debug!("Creating secret {}", admin_user_secret_name);
            let mut secret_data: BTreeMap<String, String> = BTreeMap::new();
            secret_data.insert(SECRET_KEY_URL.to_string(), management_url);

            {
                let mut rng = rand::thread_rng();

                let username_postfix = Alphanumeric.sample_string(&mut rng, 16);

                secret_data.insert(
                    SECRET_KEY_USERNAME.to_string(),
                    format!("managed_admin_{username_postfix}"),
                );

                let password = Alphanumeric.sample_string(&mut rng, 50);
                secret_data.insert(SECRET_KEY_PASSWORD.to_string(), password);
            }

            let admin_user_secret = Secret {
                metadata: ObjectMeta {
                    name: Some(admin_user_secret_name.clone()),
                    owner_references: Some(vec![self.owner_reference.clone()]),
                    labels: Some(self.get_labels()),
                    ..default()
                },
                string_data: Some(secret_data),
                ..default()
            };

            secret_api.create(&default(), &admin_user_secret).await?;
            debug!("Secret {} created", admin_user_secret_name);
        }
        Ok(admin_user_secret_name)
    }
}
