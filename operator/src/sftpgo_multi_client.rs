use reqwest::Url;
use sftpgo_client::SftpgoClient;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SftpgoMultiClient {
    clients: Arc<Mutex<HashMap<String, Arc<SftpgoClient>>>>,
}

impl SftpgoMultiClient {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_client(&self, id: &str, url: Url) -> Arc<SftpgoClient> {
        let mut all = self.clients.lock().await;

        if let Some(client) = all.get(id) {
            return client.clone();
        }

        let new_client = SftpgoClient::new(url);
        all.insert(id.to_string(), Arc::new(new_client));
        all.get(id).unwrap().clone()
    }
}
