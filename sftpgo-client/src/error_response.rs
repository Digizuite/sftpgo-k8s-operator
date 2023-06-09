use reqwest::{Response, StatusCode};
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct GenericResponseBody {
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Error, Debug)]
pub enum SftpgoError {
    #[error("Internal server error: {0:?}")]
    InternalServerError(GenericResponseBody),
    #[error("Bad request: {0:?}")]
    BadRequest(GenericResponseBody),
    #[error("Unauthorized: {0:?}")]
    Unauthorized(GenericResponseBody),
    #[error("Not found: {0:?}")]
    NotFound(GenericResponseBody),
    #[error("Request error: {0:?}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Url building failed: {0:?}. This is most likely a bug in the code.")]
    UrlBuildingFailed(#[from] url::ParseError),
}

pub async fn handle_response<T>(response: Response) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    match response.error_for_status_ref() {
        Ok(_) => Ok(response.json().await?),
        Err(err) => match err.status() {
            Some(StatusCode::UNAUTHORIZED) => {
                Err(SftpgoError::Unauthorized(response.json().await?))
            }
            Some(StatusCode::BAD_REQUEST) => Err(SftpgoError::Unauthorized(response.json().await?)),
            Some(StatusCode::INTERNAL_SERVER_ERROR) => {
                Err(SftpgoError::Unauthorized(response.json().await?))
            }
            _ => Err(SftpgoError::ReqwestError(err)),
        },
    }
}

pub type Result<T> = std::result::Result<T, SftpgoError>;
