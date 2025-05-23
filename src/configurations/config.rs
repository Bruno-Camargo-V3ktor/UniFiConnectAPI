use rocket::{config::SecretKey, figment::Figment};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use tokio::sync::RwLock;

use crate::ldap::ldap::LdapAttributes;

// Types
pub type ConfigApp = RwLock<ConfigApplication>;

// Structs
#[derive(Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub port: u16,
    pub workers: usize,
    pub log_level: String,
    pub keep_alive: u32,
    pub secret_key: String,
    pub files_dir: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UnifiConfig {
    pub url: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientsConfig {
    pub free_request: bool,
    pub time_connection: usize,
    pub expiration_time: Option<usize>,
    pub info: Option<ClientInfo>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClientInfo {
    pub name_validated: Option<String>,
    pub email_validated: Option<String>,
    pub phone_validated: Option<String>,
    pub fields: Vec< (String, String) >,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ApproversConfig {
    pub code_size: usize,
    pub validity_days_code: usize,
    pub just_numbers: bool,
    pub encrypted_code: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AdminsConfig {
    pub token_expirantion: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UsersConfig {
    pub registrations_open: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LdapConfig {
    pub server: String,
    pub domain: String,
    pub base_dn: String,
    pub user_service: String,
    pub attrs: LdapAttributes,
    pub password: String,
    pub approvers_search: Vec<String>,
    pub users_search: Vec<String>,
    pub admins_search: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GLPIConfig {
    pub url: String,
    pub app_token: String,
    pub authorization: String,
    pub title_ticket: String,
    pub body_titcket: String,
    pub approver_message: String,
    pub reject_message: String,
    pub open_status_ticket: usize,
    pub close_status_ticket: usize,
    pub user_request_id: usize,
    pub ticket_category_id: usize,
    pub ticket_priority_id: usize,
    pub template_solution_id: usize
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConfigApplication {
    pub server: ServerConfig,
    pub unifi: UnifiConfig,
    pub database: DatabaseConfig,
    pub clients: ClientsConfig,
    pub approvers: ApproversConfig,
    pub admins: AdminsConfig,
    pub users: UsersConfig,
    pub ldap: Option<LdapConfig>,
    pub glpi: Option<GLPIConfig>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConfigUpdate {
    pub server: Option<ServerConfig>,
    pub unifi: Option<UnifiConfig>,
    pub database: Option<DatabaseConfig>,
    pub clients: Option<ClientsConfig>,
    pub approvers: Option<ApproversConfig>,
    pub admins: Option<AdminsConfig>,
    pub users: Option<UsersConfig>,
    pub ldap: Option<LdapConfig>,
    pub glpi: Option<GLPIConfig>
}

// Impls
impl ConfigApplication {
    pub fn new() -> Self {
        let mut file = File::open(".config.json").expect("Configuration file not found");
        let mut json_str = String::new();
        file.read_to_string(&mut json_str)
            .expect("Error reading configuration file");

        let config: Self = serde_json::from_str(&json_str).expect("The settings are incorrect");
        config
    }

    pub fn save(&self) {
        let json_str = serde_json::to_string_pretty(&self).unwrap();
        let mut file = File::create(".config.json").expect("Configuration file not found");
        file.write_all(json_str.as_bytes())
            .expect("Error saving settings file");
    }

    pub fn to_rocket_config(&self) -> rocket::figment::Figment {
        let config = rocket::Config {
            address: self.server.address.parse().unwrap(),
            port: self.server.port,
            workers: self.server.workers,
            log_level: self.server.log_level.parse().unwrap(),
            keep_alive: self.server.keep_alive,
            secret_key: SecretKey::from(self.server.secret_key.as_bytes()),
            ..rocket::Config::default()
        };

        let database_url = self.database.get_formated_url();

        Figment::from(config).merge(("databases.mongodb", rocket_db_pools::Config {
            url: database_url,
            min_connections: Some(64),
            max_connections: 1024,
            connect_timeout: 5,
            idle_timeout: Some(120),
            extensions: None,
        }))
    }
}

impl DatabaseConfig {
    pub fn get_formated_url(&self) -> String {
        self.url
            .clone()
            .replacen("{}", &self.username.clone(), 1)
            .replacen("{}", &self.password.clone(), 1)
    }
}

