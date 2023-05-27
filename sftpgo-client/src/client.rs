use crate::error_response::Result;
use reqwest::{Client, Url};

pub trait SftpgoClientBase {
    fn get_client(&self) -> &Client;
    fn url_for(&self, endpoint: &str) -> Result<Url>;
}

pub struct SftpgoClient {
    client: Client,
    base_url: Url,
}

impl SftpgoClient {
    pub fn new(base_url: Url) -> Self {
        Self {
            client: Client::new(),
            base_url,
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

mod tests {
    #[test]
    fn is_send_and_sync() {
        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}
        is_send::<super::SftpgoClient>();
        is_sync::<super::SftpgoClient>();
    }
}
