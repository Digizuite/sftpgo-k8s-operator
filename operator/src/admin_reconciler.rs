use crate::reconciler::{ContextData, SftpgoResource};
use crate::user_reconciler::MapEnabled;
use crate::Error;
use async_trait::async_trait;
use crds::{ServerReference, SftpgoAdmin};
use sftpgo_client::admins::{AdminRequest, AdminResponse};
use sftpgo_client::UserStatus;

#[async_trait]
impl SftpgoResource for SftpgoAdmin {
    type Request = AdminRequest;
    type Response = AdminResponse;

    fn get_name(&self) -> &str {
        &self.spec.configuration.username
    }

    async fn get_request(
        &self,
        _context: &ContextData,
        _namespace: &String,
    ) -> Result<Self::Request, Error> {
        let admin_conf = &self.spec.configuration;

        let request = AdminRequest {
            username: admin_conf.username.clone(),
            description: admin_conf.description.clone(),
            password: admin_conf.password.clone(),
            email: admin_conf.email.clone(),
            permissions: admin_conf
                .permissions
                .iter()
                .map(|p| p.to_string())
                .collect(),
            status: admin_conf
                .enabled
                .map_or(UserStatus::Enabled, |status| status.map_enabled()),
            role: admin_conf.role.clone(),
        };

        Ok(request)
    }

    fn get_server_reference(&self) -> &ServerReference {
        &self.spec.server_reference
    }
}
