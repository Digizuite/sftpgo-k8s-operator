use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum UploadMode {
    #[default]
    Standard,
    Atomic,
    AtomicWithResumeSupport,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Common {
    pub idle_timeout: Option<i64>,
    pub upload_mode: Option<UploadMode>,
    pub actions: Option<Actions>,
    pub setstat_mode: Option<i64>,
    pub rename_mode: Option<i64>,
    pub temp_path: Option<String>,
    pub proxy_protocol: Option<Option<i64>>,
    pub proxy_allowed: Option<Vec<String>>,
    pub proxy_skipped: Option<Vec<String>>,
    pub startup_hook: Option<String>,
    pub post_connect_hook: Option<String>,
    pub post_disconnect_hook: Option<String>,
    pub data_retention_hook: Option<String>,
    pub max_total_connections: Option<i64>,
    pub max_per_host_connections: Option<i64>,
    pub allowlist_status: Option<i64>,
    pub allow_self_connections: Option<i64>,
    pub defender: Option<Defender>,
    pub rate_limiters: Option<Vec<RateLimiter>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Actions {
    pub execute_on: Option<Vec<String>>,
    pub execute_sync: Option<Vec<String>>,
    pub hook: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Defender {
    pub enabled: Option<bool>,
    pub driver: Option<String>,
    pub ban_time: Option<i64>,
    pub ban_time_increment: Option<i64>,
    pub threshold: Option<i64>,
    pub score_invalid: Option<i64>,
    pub score_valid: Option<i64>,
    pub score_limit_exceeded: Option<i64>,
    pub score_no_auth: Option<i64>,
    pub observation_time: Option<i64>,
    pub entries_soft_limit: Option<i64>,
    pub entries_hard_limit: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct RateLimiter {
    pub average: Option<i64>,
    pub period: Option<i64>,
    pub burst: Option<i64>,
    #[serde(rename = "type")]
    pub type_field: Option<i64>,
    pub protocols: Option<Vec<String>>,
    pub generate_defender_events: Option<bool>,
    pub entries_soft_limit: Option<i64>,
    pub entries_hard_limit: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Acme {
    pub domains: Option<Vec<String>>,
    pub email: Option<String>,
    pub key_type: Option<String>,
    pub certs_path: Option<String>,
    pub ca_endpoint: Option<String>,
    pub renew_days: Option<i64>,
    pub http01_challenge: Option<Http01Challenge>,
    pub tls_alpn01_challenge: Option<TlsAlpn01Challenge>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Http01Challenge {
    pub port: Option<i64>,
    pub proxy_header: Option<String>,
    pub webroot: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TlsAlpn01Challenge {
    pub port: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Sftpd {
    pub bindings: Option<Vec<SftpdBinding>>,
    pub max_auth_tries: Option<i64>,
    pub banner: Option<String>,
    pub host_keys: Option<Vec<String>>,
    pub host_certificates: Option<Vec<String>>,
    pub host_key_algorithms: Option<Vec<String>>,
    pub moduli: Option<Vec<String>>,
    pub kex_algorithms: Option<Vec<String>>,
    pub ciphers: Option<Vec<String>>,
    pub macs: Option<Vec<String>>,
    pub trusted_user_ca_keys: Option<Vec<String>>,
    pub revoked_user_certs_file: Option<String>,
    pub login_banner_file: Option<String>,
    pub enabled_ssh_commands: Option<Vec<String>>,
    pub keyboard_interactive_authentication: Option<bool>,
    pub keyboard_interactive_auth_hook: Option<String>,
    pub password_authentication: Option<bool>,
    pub folder_prefix: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SftpdBinding {
    pub port: Option<i64>,
    pub address: Option<String>,
    pub apply_proxy_config: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Ftpd {
    pub bindings: Option<Vec<FtpdBinding>>,
    pub banner: Option<String>,
    pub banner_file: Option<String>,
    pub active_transfers_port_non_20: Option<bool>,
    pub passive_port_range: Option<PassivePortRange>,
    pub disable_active_mode: Option<bool>,
    pub enable_site: Option<bool>,
    pub hash_support: Option<i64>,
    pub combine_support: Option<i64>,
    pub certificate_file: Option<String>,
    pub certificate_key_file: Option<String>,
    pub ca_certificates: Option<Vec<String>>,
    pub ca_revocation_lists: Option<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FtpdBinding {
    pub port: Option<i64>,
    pub address: Option<String>,
    pub apply_proxy_config: Option<bool>,
    pub tls_mode: Option<i64>,
    pub certificate_file: Option<String>,
    pub certificate_key_file: Option<String>,
    pub min_tls_version: Option<i64>,
    pub force_passive_ip: Option<String>,
    pub passive_ip_overrides: Option<Vec<FtpdBindingPassiveIpOverride>>,
    pub passive_host: Option<String>,
    pub client_auth_type: Option<i64>,
    pub tls_cipher_suites: Option<Vec<String>>,
    pub passive_connections_security: Option<i64>,
    pub active_connections_security: Option<i64>,
    pub debug: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct FtpdBindingPassiveIpOverride {
    pub networks: Option<Vec<String>>,
    pub ip: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PassivePortRange {
    pub start: Option<i64>,
    pub end: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Webdavd {
    pub bindings: Option<Vec<WebdavdBinding>>,
    pub certificate_file: Option<String>,
    pub certificate_key_file: Option<String>,
    pub ca_certificates: Option<Vec<String>>,
    pub ca_revocation_lists: Option<Vec<String>>,
    pub cors: Option<Cors>,
    pub cache: Option<Cache>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct WebdavdBinding {
    pub port: Option<i64>,
    pub address: Option<String>,
    pub enable_https: Option<bool>,
    pub certificate_file: Option<String>,
    pub certificate_key_file: Option<String>,
    pub min_tls_version: Option<i64>,
    pub client_auth_type: Option<i64>,
    pub tls_cipher_suites: Option<Vec<String>>,
    pub prefix: Option<String>,
    pub proxy_allowed: Option<Vec<String>>,
    pub client_ip_proxy_header: Option<String>,
    pub client_ip_header_depth: Option<i64>,
    pub disable_www_auth_header: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Cors {
    pub enabled: Option<bool>,
    pub allowed_origins: Option<Vec<String>>,
    pub allowed_methods: Option<Vec<String>>,
    pub allowed_headers: Option<Vec<String>>,
    pub exposed_headers: Option<Vec<String>>,
    pub allow_credentials: Option<bool>,
    pub max_age: Option<i64>,
    pub options_passthrough: Option<bool>,
    pub options_success_status: Option<i64>,
    pub allow_private_network: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Cache {
    pub users: Option<Users>,
    pub mime_types: Option<MimeTypes>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Users {
    pub expiration_time: Option<i64>,
    pub max_size: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MimeTypes {
    pub enabled: Option<bool>,
    pub max_size: Option<i64>,
    pub custom_mappings: Option<Vec<MimeTypesCustomMapping>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct MimeTypesCustomMapping {
    pub ext: String,
    pub mime: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DataProvider {
    pub driver: Option<String>,
    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<i64>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub sslmode: Option<i64>,
    pub disable_sni: Option<bool>,
    pub target_session_attrs: Option<String>,
    pub root_cert: Option<String>,
    pub client_cert: Option<String>,
    pub client_key: Option<String>,
    pub connection_string: Option<String>,
    pub sql_tables_prefix: Option<String>,
    pub track_quota: Option<i64>,
    pub delayed_quota_update: Option<i64>,
    pub pool_size: Option<i64>,
    pub users_base_dir: Option<String>,
    pub actions: Option<DataProviderActions>,
    pub external_auth_hook: Option<String>,
    pub external_auth_scope: Option<i64>,
    pub pre_login_hook: Option<String>,
    pub post_login_hook: Option<String>,
    pub post_login_scope: Option<i64>,
    pub check_password_hook: Option<String>,
    pub check_password_scope: Option<i64>,
    pub password_hashing: Option<PasswordHashing>,
    pub password_validation: Option<PasswordValidation>,
    pub password_caching: Option<bool>,
    pub update_mode: Option<i64>,
    pub create_default_admin: Option<bool>,
    pub naming_rules: Option<i64>,
    pub is_shared: Option<i64>,
    pub node: Option<Node>,
    pub backups_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum DataProviderActionsExecuteOn {
    #[serde(rename = "add")]
    Add,
    #[serde(rename = "update")]
    Update,
    #[serde(rename = "delete")]
    Delete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum DataProviderActionsExecuteFor {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "folder")]
    Folder,
    #[serde(rename = "group")]
    Group,
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "api_key")]
    ApiKey,
    #[serde(rename = "share")]
    Share,
    #[serde(rename = "event_action")]
    EventAction,
    #[serde(rename = "event_rule")]
    EventRule,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DataProviderActions {
    pub execute_on: Option<Vec<DataProviderActionsExecuteOn>>,
    pub execute_for: Option<Vec<DataProviderActionsExecuteFor>>,
    pub hook: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PasswordHashing {
    pub bcrypt_options: Option<BcryptOptions>,
    pub argon2_options: Option<Argon2Options>,
    pub algo: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BcryptOptions {
    pub cost: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Argon2Options {
    pub memory: Option<i64>,
    pub iterations: Option<i64>,
    pub parallelism: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct PasswordValidation {
    pub admins: Option<Admins>,
    pub users: Option<Users2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Admins {
    pub min_entropy: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Users2 {
    pub min_entropy: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Node {
    pub host: Option<String>,
    pub port: Option<i64>,
    pub proto: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Httpd {
    pub bindings: Option<Vec<HttpdBinding>>,
    pub templates_path: Option<String>,
    pub static_files_path: Option<String>,
    pub openapi_path: Option<String>,
    pub web_root: Option<String>,
    pub certificate_file: Option<String>,
    pub certificate_key_file: Option<String>,
    pub ca_certificates: Option<Vec<String>>,
    pub ca_revocation_lists: Option<Vec<String>>,
    pub signing_passphrase: Option<String>,
    pub token_validation: Option<i64>,
    pub max_upload_file_size: Option<i64>,
    pub cors: Option<HttpdCors>,
    pub setup: Option<Setup>,
    pub hide_support_link: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HttpdBinding {
    pub port: Option<i64>,
    pub address: Option<String>,
    pub enable_web_admin: Option<bool>,
    pub enable_web_client: Option<bool>,
    pub enable_rest_api: Option<bool>,
    pub enabled_login_methods: Option<i64>,
    pub enable_https: Option<bool>,
    pub certificate_file: Option<String>,
    pub certificate_key_file: Option<String>,
    pub min_tls_version: Option<i64>,
    pub client_auth_type: Option<i64>,
    pub tls_cipher_suites: Option<Vec<String>>,
    pub proxy_allowed: Option<Vec<String>>,
    pub client_ip_proxy_header: Option<String>,
    pub client_ip_header_depth: Option<i64>,
    pub hide_login_url: Option<i64>,
    pub render_openapi: Option<bool>,
    pub web_client_integrations: Option<Vec<HttpdBindingWebClientIntegration>>,
    pub oidc: Option<Oidc>,
    pub security: Option<Security>,
    pub branding: Option<Branding>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HttpdBindingWebClientIntegration {
    pub file_extensions: Vec<String>,
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Oidc {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub config_url: Option<String>,
    pub redirect_base_url: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub username_field: Option<String>,
    pub role_field: Option<String>,
    pub implicit_roles: Option<bool>,
    pub custom_fields: Option<Vec<String>>,
    pub insecure_skip_signature_check: Option<bool>,
    pub debug: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Security {
    pub enabled: Option<bool>,
    pub allowed_hosts: Option<Vec<String>>,
    pub allowed_hosts_are_regex: Option<bool>,
    pub hosts_proxy_headers: Option<Vec<String>>,
    pub https_redirect: Option<bool>,
    pub https_host: Option<String>,
    pub https_proxy_headers: Option<Vec<SecurityHttpsProxyHeader>>,
    pub sts_seconds: Option<i64>,
    pub sts_include_subdomains: Option<bool>,
    pub sts_preload: Option<bool>,
    pub content_type_nosniff: Option<bool>,
    pub content_security_policy: Option<String>,
    pub permissions_policy: Option<String>,
    pub cross_origin_opener_policy: Option<String>,
    pub expect_ct_header: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SecurityHttpsProxyHeader {
    pub key: String,
    pub value: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Branding {
    pub web_admin: Option<WebAdmin>,
    pub web_client: Option<WebClient>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct WebAdmin {
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub favicon_path: Option<String>,
    pub logo_path: Option<String>,
    pub login_image_path: Option<String>,
    pub disclaimer_name: Option<String>,
    pub disclaimer_path: Option<String>,
    pub default_css: Option<String>,
    pub extra_css: Option<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct WebClient {
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub favicon_path: Option<String>,
    pub logo_path: Option<String>,
    pub login_image_path: Option<String>,
    pub disclaimer_name: Option<String>,
    pub disclaimer_path: Option<String>,
    pub default_css: Option<String>,
    pub extra_css: Option<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HttpdCors {
    pub enabled: Option<bool>,
    pub allowed_origins: Option<Vec<String>>,
    pub allowed_methods: Option<Vec<String>>,
    pub allowed_headers: Option<Vec<String>>,
    pub exposed_headers: Option<Vec<String>>,
    pub allow_credentials: Option<bool>,
    pub max_age: Option<i64>,
    pub options_passthrough: Option<bool>,
    pub options_success_status: Option<i64>,
    pub allow_private_network: Option<bool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Setup {
    pub installation_code: Option<String>,
    pub installation_code_hint: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Telemetry {
    pub bind_port: Option<i64>,
    pub bind_address: Option<String>,
    pub enable_profiler: Option<bool>,
    pub auth_user_file: Option<String>,
    pub certificate_file: Option<String>,
    pub certificate_key_file: Option<String>,
    pub min_tls_version: Option<i64>,
    pub tls_cipher_suites: Option<Vec<String>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HttpClient {
    pub timeout: Option<i64>,
    pub retry_wait_min: Option<i64>,
    pub retry_wait_max: Option<i64>,
    pub retry_max: Option<i64>,
    pub ca_certificates: Option<Vec<String>>,
    pub certificates: Option<Vec<HttpClientCertificate>>,
    pub skip_tls_verify: Option<bool>,
    pub headers: Option<Vec<HttpClientHeaders>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HttpClientCertificate {
    pub cert: String,
    pub key: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct HttpClientHeaders {
    pub key: String,
    pub value: String,
    pub url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Command {
    pub timeout: Option<i64>,
    pub env: Option<Vec<String>>,
    pub commands: Option<Vec<CommandCommand>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CommandCommand {
    pub path: String,
    pub timeout: Option<i64>,
    pub env: Option<Vec<String>>,
    pub args: Option<Vec<String>>,
    pub hook: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Kms {
    pub secrets: Option<Secrets>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Secrets {
    pub url: Option<String>,
    pub master_key: Option<String>,
    pub master_key_path: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Mfa {
    pub totp: Option<Vec<Totp>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Totp {
    pub name: Option<String>,
    pub issuer: Option<String>,
    pub algo: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Smtp {
    pub host: Option<String>,
    pub port: Option<i64>,
    pub from: Option<String>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub auth_type: Option<i64>,
    pub encryption: Option<i64>,
    pub domain: Option<String>,
    pub templates_path: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SftpgoConfiguration {
    pub common: Option<Common>,
    pub acme: Option<Acme>,
    pub sftpd: Option<Sftpd>,
    pub ftpd: Option<Ftpd>,
    pub webdavd: Option<Webdavd>,
    pub data_provider: Option<DataProvider>,
    pub httpd: Option<Httpd>,
    pub telemetry: Option<Telemetry>,
    pub http: Option<HttpClient>,
    pub command: Option<Command>,
    pub kms: Option<Kms>,
    pub mfa: Option<Mfa>,
    pub smtp: Option<Smtp>,
}

#[derive(CustomResource, Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[kube(
    group = "sftpgo.zlepper.dk",
    version = "v1alpha1",
    kind = "SftpgoServer",
    plural = "sftpgoservers",
    derive = "PartialEq",
    namespaced
)]
pub struct SftpgoServerSpec {
    pub configuration: Option<SftpgoConfiguration>,
    pub replicas: Option<i32>,
    pub image: Option<String>,
    pub labels: Option<BTreeMap<String, String>>,
}
