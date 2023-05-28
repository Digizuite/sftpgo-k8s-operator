use crate::consts::{SECRET_KEY_PASSWORD, SECRET_KEY_URL, SECRET_KEY_USERNAME};
use crate::reconciler::{ContextData, Error};
use crds::ServerReference;
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::ByteString;
use kube::Api;
use reqwest::Url;
use sftpgo_client::{
    AuthorizedSftpgoClient, RefreshableAdminAuthContext, SftpgoClient, UsersClient,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

pub struct SftpgoMultiClient {
    clients: Arc<Mutex<HashMap<String, Arc<KnownSftpgoClient>>>>,
}

impl SftpgoMultiClient {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_client(&self, id: &str, url: Url) -> Arc<KnownSftpgoClient> {
        let mut all = self.clients.lock().await;

        if let Some(client) = all.get(id) {
            return client.clone();
        }

        let new_client = KnownSftpgoClient::new(url);
        all.insert(id.to_string(), Arc::new(new_client));
        all.get(id).unwrap().clone()
    }
}

#[derive(Clone)]
pub struct KnownSftpgoClient {
    client: SftpgoClient,
    authorized_clients: Arc<
        RwLock<HashMap<String, AuthorizedSftpgoClient<RefreshableAdminAuthContext<SftpgoClient>>>>,
    >,
}

impl KnownSftpgoClient {
    fn new(url: Url) -> KnownSftpgoClient {
        KnownSftpgoClient {
            client: SftpgoClient::new(url),
            authorized_clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_authorized_client<'a>(
        &self,
        username: &str,
        password: &str,
    ) -> Result<AuthorizedSftpgoClient<RefreshableAdminAuthContext<SftpgoClient>>, Error> {
        {
            let all = self.authorized_clients.read().await;

            if let Some(client) = all.get(username) {
                return Ok(client.clone());
            }
        }

        {
            let mut all = self.authorized_clients.write().await;

            let ctx = RefreshableAdminAuthContext::new(
                username.to_string(),
                password.to_string(),
                self.client.clone(),
            )
            .await?;

            let new_client = self.client.with_auth_context(ctx);
            all.insert(username.to_string(), new_client.clone());
            Ok(new_client)
        }
    }
}

pub async fn get_api_client(
    server_ref: &ServerReference,
    context: &ContextData,
    namespace: &String,
) -> Result<Box<dyn UsersClient>, Error> {
    if let Some(connection_secret) = &server_ref.connection_secret {
        if server_ref.name.is_some() || server_ref.namespace.is_some() {
            return Err(Error::UserInput(
                "Both connectionSecret and name/namespace are set. Only one set can be specified"
                    .to_string(),
            ));
        }

        let secret_namespace = connection_secret.namespace.as_ref().unwrap_or(namespace);
        let secret_name = &connection_secret.name;

        Ok(get_admin_secret(context, secret_namespace, secret_name).await?)
    } else if let Some(name) = &server_ref.name {
        let target_namespace = server_ref.namespace.as_ref().unwrap_or(namespace);

        let admin_user_secret_name = format!("{}-admin-user", name);

        Ok(get_admin_secret(context, target_namespace, &admin_user_secret_name).await?)
    } else {
        Err(Error::UserInput(
            "Either connectionSecret or name/namespace must be set".to_string(),
        ))
    }
}

async fn get_admin_secret(
    context: &ContextData,
    secret_namespace: &str,
    secret_name: &str,
) -> Result<Box<dyn UsersClient>, Error> {
    let secret_api: Api<Secret> =
        Api::namespaced(context.kubernetes_client.clone(), secret_namespace);

    if let Some(secret) = secret_api.get_opt(secret_name).await? {
        debug!("Secret {} found", secret_name);

        if let Some(sd) = &secret.data {
            let ((url, username), password) = sd
                .get(SECRET_KEY_URL)
                .zip(sd.get(SECRET_KEY_USERNAME))
                .zip(sd.get(SECRET_KEY_PASSWORD))
                .ok_or(Error::UserInput(format!(
                    "Secret {} does not contain all required keys. Expected '{}', '{}' and '{}'",
                    secret_name, SECRET_KEY_USERNAME, SECRET_KEY_URL, SECRET_KEY_PASSWORD
                )))?;

            let url = parse_secret_value(url)?;
            let username = parse_secret_value(username)?;
            let password = parse_secret_value(password)?;

            let u = Url::parse(url).map_err(|e| {
                Error::UserInput(format!(
                    "Secret {} contains invalid URL: {}",
                    secret_name, e
                ))
            })?;
            let c = context
                .sftpgo_client
                .get_client(&secret.metadata.uid.unwrap(), u)
                .await;

            let authorized_client = c.get_authorized_client(username, password).await?;

            Ok(Box::new(authorized_client))
        } else {
            Err(Error::UserInput(format!(
                "Secret {} does not contain stringData",
                secret_name
            )))
        }
    } else {
        Err(Error::UserInput(format!(
            "Secret {} not found in namespace {}",
            secret_name, secret_namespace
        )))
    }
}

fn parse_secret_value(value: &ByteString) -> Result<&str, Error> {
    std::str::from_utf8(&value.0)
        .map_err(|e| Error::UserInput(format!("Secret contains invalid UTF-8: {}", e)))
}
