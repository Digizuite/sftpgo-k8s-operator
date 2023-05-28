use crate::auth::AuthContext;
use crate::error_response::Result;
use reqwest::{Client, Url};
use std::sync::Arc;

pub trait SftpgoClientBase {
    fn get_client(&self) -> &Client;
    fn url_for(&self, endpoint: &str) -> Result<Url>;
}

pub trait AuthorizedSftpgoClientBase: SftpgoClientBase {
    fn get_auth_context(&self) -> &dyn AuthContext;
}

#[derive(Clone)]
pub struct SftpgoClient {
    client: Client,
    base_url: Arc<Url>,
}

impl SftpgoClient {
    pub fn new(base_url: Url) -> Self {
        Self {
            client: Client::new(),
            base_url: Arc::new(base_url),
        }
    }

    pub fn with_auth_context<TAuthContext>(
        &self,
        auth_context: TAuthContext,
    ) -> AuthorizedSftpgoClient<TAuthContext>
    where
        TAuthContext: AuthContext + Clone,
    {
        AuthorizedSftpgoClient {
            client: self.clone(),
            auth_context,
        }
    }
}

impl SftpgoClientBase for SftpgoClient {
    fn get_client(&self) -> &Client {
        &self.client
    }

    fn url_for(&self, endpoint: &str) -> Result<Url> {
        Ok(self.base_url.join(endpoint)?)
    }
}

#[derive(Clone)]
pub struct AuthorizedSftpgoClient<TAuthContext>
where
    TAuthContext: AuthContext + Clone,
{
    client: SftpgoClient,
    auth_context: TAuthContext,
}

impl<TAuthContext> SftpgoClientBase for AuthorizedSftpgoClient<TAuthContext>
where
    TAuthContext: AuthContext + Clone,
{
    fn get_client(&self) -> &Client {
        self.client.get_client()
    }

    fn url_for(&self, endpoint: &str) -> Result<Url> {
        self.client.url_for(endpoint)
    }
}

impl<TAuthContext> AuthorizedSftpgoClientBase for AuthorizedSftpgoClient<TAuthContext>
where
    TAuthContext: AuthContext + Clone,
{
    fn get_auth_context(&self) -> &dyn AuthContext {
        &self.auth_context
    }
}

mod tests {
    #[test]
    fn is_send_and_sync() {
        use crate::{RefreshableAdminAuthContext, SftpgoClient};

        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}
        is_send::<super::SftpgoClient>();
        is_sync::<super::SftpgoClient>();
        is_send::<super::AuthorizedSftpgoClient<RefreshableAdminAuthContext<SftpgoClient>>>();
        is_sync::<super::AuthorizedSftpgoClient<RefreshableAdminAuthContext<SftpgoClient>>>();
    }
}
