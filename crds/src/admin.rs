use crate::sftpgo_server_reference::ServerReference;
use crate::{SftpgoStatus, SftpgoUserStatus};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AdminPermission {
    #[default]
    All,
    AddUsers,
    EditUsers,
    DelUsers,
    ViewUsers,
    ViewConns,
    CloseConns,
    ViewStatus,
    ManageAdmins,
    ManageGroups,
    ManageApikeys,
    QuotaScans,
    ManageSystem,
    ManageDefender,
    ViewDefender,
    RetentionChecks,
    MetadataChecks,
    ViewEvents,
    ManageEventRules,
    ManageRoles,
    ManageIpLists,
}

impl ToString for AdminPermission {
    fn to_string(&self) -> String {
        match self {
            AdminPermission::All => "*".to_string(),
            AdminPermission::AddUsers => "add_users".to_string(),
            AdminPermission::EditUsers => "edit_users".to_string(),
            AdminPermission::DelUsers => "del_users".to_string(),
            AdminPermission::ViewUsers => "view_users".to_string(),
            AdminPermission::ViewConns => "view_conns".to_string(),
            AdminPermission::CloseConns => "close_conns".to_string(),
            AdminPermission::ViewStatus => "view_status".to_string(),
            AdminPermission::ManageAdmins => "manage_admins".to_string(),
            AdminPermission::ManageGroups => "manage_groups".to_string(),
            AdminPermission::ManageApikeys => "manage_apikeys".to_string(),
            AdminPermission::QuotaScans => "quota_scans".to_string(),
            AdminPermission::ManageSystem => "manage_system".to_string(),
            AdminPermission::ManageDefender => "manage_defender".to_string(),
            AdminPermission::ViewDefender => "view_defender".to_string(),
            AdminPermission::RetentionChecks => "retention_checks".to_string(),
            AdminPermission::MetadataChecks => "metadata_checks".to_string(),
            AdminPermission::ViewEvents => "view_events".to_string(),
            AdminPermission::ManageEventRules => "manage_event_rules".to_string(),
            AdminPermission::ManageRoles => "manage_roles".to_string(),
            AdminPermission::ManageIpLists => "manage_ip_lists".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SftpgoAdminConfiguration {
    /// The username of the user
    pub username: String,
    /// optional description, for example the admin full name
    pub description: Option<String>,
    /// Password of the user. Changes to this field will not propagate to the user after creation as we have
    /// no way of retrieving the password from the server.
    pub password: String,
    pub enabled: Option<SftpgoUserStatus>,
    pub email: Option<String>,
    pub permissions: Vec<AdminPermission>,
    /// If set the admin can only administer users with the same role. Role admins cannot have the
    /// following permissions: "manage_admins", "manage_apikeys", "manage_system",
    /// "manage_event_rules", "manage_roles", "manage_ip_lists"
    pub role: Option<String>,
}

#[derive(CustomResource, Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[kube(
    group = "sftpgo.zlepper.dk",
    version = "v1alpha1",
    kind = "SftpgoAdmin",
    plural = "sftpgoadmins",
    derive = "PartialEq",
    status = "SftpgoAdminResourceStatus",
    namespaced
)]
#[serde(rename_all = "camelCase")]
pub struct SftpgoAdminSpec {
    pub configuration: SftpgoAdminConfiguration,

    #[serde(rename = "sftpgoServerReference")]
    pub server_reference: ServerReference,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema, Default)]
pub struct SftpgoAdminResourceStatus {
    pub last_username: String,
    pub admin_id: Option<i32>,
}

impl SftpgoStatus for SftpgoAdminResourceStatus {
    fn get_last_name(&self) -> &str {
        &self.last_username
    }

    fn set_last_name(&mut self, name: &str) {
        self.last_username = name.to_string();
    }

    fn get_id(&self) -> Option<i32> {
        self.admin_id
    }

    fn set_id(&mut self, id: Option<i32>) {
        self.admin_id = id;
    }
}
