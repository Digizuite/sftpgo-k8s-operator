use crate::{handle_response, AuthorizedSftpgoClientBase, GenericResponseBody};
use async_trait::async_trait;
use reqwest::header::AUTHORIZATION;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use url::Url;

pub trait Named {
    fn name(&self) -> &str;
}

pub trait Existing {
    fn name(&self) -> &str;
    fn id(&self) -> i32;
}

pub trait Creates<TResponse>: Named {}
pub trait CreatedFrom<TRequest>: Existing {}

impl<TRequest: Named, TResponse: Existing> CreatedFrom<TRequest> for TResponse {}

#[async_trait]
pub trait SftpgoRestClient<TRequest, TResponse>: Send + Sync
where
    TRequest: Serialize + Sync + Creates<TResponse>,
    TResponse: for<'de> Deserialize<'de> + CreatedFrom<TRequest>,
{
    async fn create(&self, item: &TRequest) -> crate::Result<TResponse>;
    async fn update(&self, item: &TRequest) -> crate::Result<GenericResponseBody>;
    async fn delete(&self, name: &str) -> crate::Result<()>;
    async fn get(&self, name: &str) -> crate::Result<Option<TResponse>>;
}

pub trait EasyRestSftpgoClient<TRequest, TResponse> {
    fn get_url(&self, path: Option<&str>) -> crate::Result<Url>;
}

#[async_trait]
impl<Client, TRequest, TResponse> SftpgoRestClient<TRequest, TResponse> for Client
where
    Client: AuthorizedSftpgoClientBase + EasyRestSftpgoClient<TRequest, TResponse> + Send + Sync,
    TRequest: Serialize + Sync + Creates<TResponse>,
    TResponse: for<'de> Deserialize<'de> + CreatedFrom<TRequest>,
{
    async fn create(&self, item: &TRequest) -> crate::Result<TResponse> {
        let url = self.get_url(None)?;

        let auth_header_value = self.get_auth_context().get_auth_header_value().await?;
        let res = self
            .get_client()
            .post(url)
            .header(AUTHORIZATION, auth_header_value)
            .json(&item)
            .send()
            .await?;

        handle_response(res).await
    }

    async fn update(&self, item: &TRequest) -> crate::Result<GenericResponseBody> {
        let url = self.get_url(Some(item.name()))?;

        let auth_header_value = self.get_auth_context().get_auth_header_value().await?;
        let res = self
            .get_client()
            .put(url)
            .header(AUTHORIZATION, auth_header_value)
            .json(&item)
            .send()
            .await?;

        handle_response(res).await
    }

    async fn delete(&self, name: &str) -> crate::Result<()> {
        let url = self.get_url(Some(name))?;

        let auth_header_value = self.get_auth_context().get_auth_header_value().await?;
        let res = self
            .get_client()
            .delete(url)
            .header(AUTHORIZATION, auth_header_value)
            .send()
            .await?;

        if res.status() == StatusCode::NOT_FOUND || res.status() == StatusCode::OK {
            return Ok(());
        }

        handle_response(res).await
    }

    async fn get(&self, name: &str) -> crate::Result<Option<TResponse>> {
        let url = self.get_url(Some(name))?;

        let auth_header_value = self.get_auth_context().get_auth_header_value().await?;
        let res = self
            .get_client()
            .get(url)
            .header(AUTHORIZATION, auth_header_value)
            .send()
            .await?;

        if res.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        handle_response(res).await
    }
}
